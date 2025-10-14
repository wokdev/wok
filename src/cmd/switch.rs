use anyhow::*;
use std::io::Write;
use std::result::Result::Ok;

use crate::{config, repo};

pub fn switch<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
    create: bool,
    all: bool,
    branch_name: Option<&str>,
    target_repos: &[std::path::PathBuf],
) -> Result<bool> {
    let mut config_updated = false;

    // Determine the target branch
    let target_branch = match branch_name {
        Some(name) => name.to_string(),
        None => umbrella.head.clone(),
    };

    // Determine which repos to switch
    let repos_to_switch: Vec<config::Repo> = if all {
        // Switch all configured repos, skipping those opted out unless explicitly targeted
        wok_config
            .repos
            .iter()
            .filter(|config_repo| {
                !config_repo.is_skipped_for("switch")
                    || target_repos.contains(&config_repo.path)
            })
            .cloned()
            .collect()
    } else if !target_repos.is_empty() {
        // Switch only specified repos
        wok_config
            .repos
            .iter()
            .filter(|config_repo| target_repos.contains(&config_repo.path))
            .cloned()
            .collect()
    } else {
        // Switch repos that match the current main repo branch
        wok_config
            .repos
            .iter()
            .filter(|config_repo| config_repo.head == umbrella.head)
            .cloned()
            .collect()
    };

    if repos_to_switch.is_empty() {
        writeln!(stdout, "No repositories to switch")?;
        return Ok(config_updated);
    }

    writeln!(
        stdout,
        "Switching {} repositories to branch '{}'...",
        repos_to_switch.len(),
        target_branch
    )?;

    // Switch each repo
    for config_repo in &repos_to_switch {
        if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
            match switch_repo(subrepo, &target_branch, create) {
                Ok(result) => {
                    config_updated |= wok_config.set_repo_head(
                        config_repo.path.as_path(),
                        &target_branch,
                    );

                    match result {
                        SwitchResult::Switched => {
                            writeln!(
                                stdout,
                                "- '{}': switched to '{}'",
                                config_repo.path.display(),
                                target_branch
                            )?;
                        },
                        SwitchResult::Created => {
                            writeln!(
                                stdout,
                                "- '{}': created and switched to '{}'",
                                config_repo.path.display(),
                                target_branch
                            )?;
                        },
                        SwitchResult::AlreadyOnBranch => {
                            writeln!(
                                stdout,
                                "- '{}': already on '{}'",
                                config_repo.path.display(),
                                target_branch
                            )?;
                        },
                    };
                },
                Err(e) => {
                    writeln!(
                        stdout,
                        "- '{}': failed to switch to '{}' - {}",
                        config_repo.path.display(),
                        target_branch,
                        e
                    )?;
                },
            }
        }
    }

    // Perform lock operation on switched repos
    writeln!(stdout, "Locking submodule state...")?;
    lock_switched_repos(umbrella, &repos_to_switch)?;

    writeln!(
        stdout,
        "Successfully switched and locked {} repositories",
        repos_to_switch.len()
    )?;
    Ok(config_updated)
}

#[derive(Debug, Clone, PartialEq)]
enum SwitchResult {
    Switched,
    Created,
    AlreadyOnBranch,
}

fn switch_repo(
    repo: &repo::Repo,
    branch_name: &str,
    create: bool,
) -> Result<SwitchResult> {
    // Check if we're already on the target branch
    if repo_on_branch(repo, branch_name)? {
        return Ok(SwitchResult::AlreadyOnBranch);
    }

    // Try to switch to the branch
    match repo.switch(branch_name) {
        Ok(_) => Ok(SwitchResult::Switched),
        Err(_) => {
            if create {
                // Try to create the branch
                create_and_switch_branch(repo, branch_name)?;
                Ok(SwitchResult::Created)
            } else {
                Err(anyhow!(
                    "Branch '{}' does not exist and --create not specified",
                    branch_name
                ))
            }
        },
    }
}

fn create_and_switch_branch(repo: &repo::Repo, branch_name: &str) -> Result<()> {
    // Get the current commit
    let head = repo.git_repo.head()?;
    let current_commit = head.peel_to_commit()?;

    // Create the new branch
    let _branch_ref = repo.git_repo.branch(branch_name, &current_commit, false)?;

    // Switch to the new branch
    repo.git_repo
        .set_head(&format!("refs/heads/{}", branch_name))?;
    repo.git_repo.checkout_head(None)?;

    Ok(())
}

fn repo_on_branch(repo: &repo::Repo, branch_name: &str) -> Result<bool> {
    if repo
        .git_repo
        .head_detached()
        .with_context(|| format!("Cannot determine head state for repo at `{}`", repo.work_dir.display()))?
    {
        return Ok(false);
    }

    let current = repo
        .git_repo
        .head()
        .with_context(|| format!("Cannot find the head branch for repo at `{}`", repo.work_dir.display()))?
        .shorthand()
        .with_context(|| {
            format!(
                "Cannot resolve the head reference for repo at `{}`",
                repo.work_dir.display()
            )
        })?
        .to_owned();

    Ok(current == branch_name)
}

fn lock_switched_repos(
    umbrella: &repo::Repo,
    _switched_repos: &[config::Repo],
) -> Result<()> {
    // Add all submodule changes to the index
    let mut index = umbrella.git_repo.index()?;
    for submodule in umbrella.git_repo.submodules()? {
        let submodule_path = submodule.path();

        // Only add submodules that have a head (are initialized)
        if let Some(_submodule_oid) = submodule.head_id() {
            // Add the submodule entry to the index
            index.add_path(submodule_path)?;
        }
    }
    index.write()?;

    // Commit the submodule state
    let commit_message = "Switch and lock submodule state";
    let signature = umbrella.git_repo.signature()?;
    let tree_id = umbrella.git_repo.index()?.write_tree()?;
    let tree = umbrella.git_repo.find_tree(tree_id)?;

    let head_ref = umbrella.git_repo.head()?;
    let parent_commit = head_ref.peel_to_commit()?;

    umbrella.git_repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(())
}
