use anyhow::*;
use std::io::Write;
use std::result::Result::Ok;

use crate::{config, repo};

#[allow(clippy::too_many_arguments)]
pub fn switch<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    config_path: &std::path::Path,
    stdout: &mut W,
    create: bool,
    all: bool,
    branch_name: &str,
    target_repos: &[std::path::PathBuf],
) -> Result<bool> {
    let mut config_updated = false;
    let mut submodule_changed = false;

    let umbrella_branch = branch_name.to_string();

    let switch_plan: Vec<SwitchPlanItem> = wok_config
        .repos
        .iter()
        .filter_map(|config_repo| {
            let explicitly_targeted = target_repos.contains(&config_repo.path);
            if config_repo.is_skipped_for("switch") && !explicitly_targeted {
                return None;
            }

            let desired_branch = if config_repo.head.trim().is_empty() {
                umbrella_branch.clone()
            } else {
                config_repo.head.clone()
            };
            let forced = all || explicitly_targeted;
            let effective_branch = if forced {
                umbrella_branch.clone()
            } else {
                desired_branch.clone()
            };

            Some(SwitchPlanItem {
                config_repo: config_repo.clone(),
                effective_branch,
                forced,
            })
        })
        .collect();

    if switch_plan.is_empty() {
        writeln!(stdout, "No repositories to switch")?;
        return Ok(config_updated);
    }

    writeln!(
        stdout,
        "Switching {} repositories for umbrella branch '{}'...",
        switch_plan.len(),
        umbrella_branch
    )?;

    // Switch each repo
    for plan_item in &switch_plan {
        let config_repo = &plan_item.config_repo;
        if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
            match switch_repo(subrepo, &plan_item.effective_branch, create) {
                Ok(result) => {
                    if plan_item.forced {
                        config_updated |= wok_config.set_repo_head(
                            config_repo.path.as_path(),
                            &umbrella_branch,
                        );
                    }

                    match result {
                        SwitchResult::Switched => {
                            writeln!(
                                stdout,
                                "- '{}': switched to '{}'",
                                config_repo.path.display(),
                                plan_item.effective_branch
                            )?;
                            submodule_changed = true;
                        },
                        SwitchResult::Created => {
                            writeln!(
                                stdout,
                                "- '{}': created and switched to '{}'",
                                config_repo.path.display(),
                                plan_item.effective_branch
                            )?;
                            submodule_changed = true;
                        },
                        SwitchResult::AlreadyOnBranch => {
                            writeln!(
                                stdout,
                                "- '{}': already on '{}'",
                                config_repo.path.display(),
                                plan_item.effective_branch
                            )?;
                        },
                    };
                },
                Err(e) => {
                    writeln!(
                        stdout,
                        "- '{}': failed to switch to '{}' - {}",
                        config_repo.path.display(),
                        plan_item.effective_branch,
                        e
                    )?;
                },
            }
        }
    }

    if config_updated {
        wok_config
            .save(config_path)
            .context("Cannot save updated wok file before locking")?;
    }

    if submodule_changed || config_updated {
        // Perform lock operation on switched repos
        writeln!(stdout, "Locking workspace state...")?;
        let switched_repos: Vec<config::Repo> = switch_plan
            .iter()
            .map(|plan| plan.config_repo.clone())
            .collect();
        lock_switched_repos(umbrella, config_path, &switched_repos)?;

        writeln!(
            stdout,
            "Successfully switched and locked {} repositories",
            switch_plan.len()
        )?;
    } else {
        writeln!(stdout, "No workspace changes detected; skipping lock")?;
        writeln!(
            stdout,
            "Successfully processed {} repositories",
            switch_plan.len()
        )?;
    }

    Ok(config_updated)
}

#[derive(Debug, Clone)]
struct SwitchPlanItem {
    config_repo: config::Repo,
    effective_branch: String,
    forced: bool,
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
    let on_branch = repo_on_branch(repo, branch_name)?;
    if on_branch {
        repo.refresh_worktree_force()?;
        return Ok(SwitchResult::AlreadyOnBranch);
    }

    // Try to switch to the branch
    match repo.switch_force(branch_name) {
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
    if repo.git_repo.head_detached().with_context(|| {
        format!(
            "Cannot determine head state for repo at `{}`",
            repo.work_dir.display()
        )
    })? {
        return Ok(false);
    }

    let current = repo
        .git_repo
        .head()
        .with_context(|| {
            format!(
                "Cannot find the head branch for repo at `{}`",
                repo.work_dir.display()
            )
        })?
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
    config_path: &std::path::Path,
    switched_repos: &[config::Repo],
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

    let wokfile_path =
        config_path
            .strip_prefix(&umbrella.work_dir)
            .with_context(|| {
                format!(
                    "Wokfile must be inside umbrella repo to be committed: `{}`",
                    config_path.display()
                )
            })?;
    index.add_path(wokfile_path)?;
    index.write()?;

    // Check if there are any changes to commit
    let signature = umbrella.git_repo.signature()?;
    let tree_id = umbrella.git_repo.index()?.write_tree()?;
    let tree = umbrella.git_repo.find_tree(tree_id)?;

    let head_ref = umbrella.git_repo.head()?;
    let parent_commit = head_ref.peel_to_commit()?;
    let parent_tree = parent_commit.tree()?;

    if tree.id() == parent_tree.id() {
        return Ok(());
    }

    // Build commit message with switched submodule info
    let (commit_message, _changed_submodules) =
        build_switch_commit_message(umbrella, &parent_tree, &tree, switched_repos)?;

    umbrella.git_repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(())
}

/// Build a commit message for switch operation showing which repos were switched.
/// Returns (commit_message, changed_submodules_list)
fn build_switch_commit_message(
    umbrella: &repo::Repo,
    parent_tree: &git2::Tree,
    index_tree: &git2::Tree,
    switched_repos: &[config::Repo],
) -> Result<(String, Vec<(String, String)>)> {
    // Get diff between parent tree and staged index
    let diff = umbrella.git_repo.diff_tree_to_tree(
        Some(parent_tree),
        Some(index_tree),
        None,
    )?;

    let mut changed_submodules = Vec::new();

    // Build a map of switched repos for quick lookup
    let switched_paths: std::collections::HashSet<_> = switched_repos
        .iter()
        .map(|r| r.path.to_string_lossy().to_string())
        .collect();

    // Iterate through changed files in the diff
    for delta in diff.deltas() {
        if let Some(file_path) = delta.new_file().path()
            && let Some(file_path_str) = file_path.to_str()
        {
            match umbrella.git_repo.find_submodule(file_path_str) {
                std::result::Result::Ok(submodule) => {
                    let submodule_name = submodule.path().to_string_lossy().to_string();

                    // Only include submodules that were actually switched
                    if switched_paths.contains(&submodule_name) {
                        let submodule_repo_path =
                            umbrella.work_dir.join(submodule.path());
                        match git2::Repository::open(&submodule_repo_path) {
                            std::result::Result::Ok(subrepo_git) => {
                                match subrepo_git.head() {
                                    std::result::Result::Ok(head_ref) => {
                                        if let Some(branch_name) = head_ref.shorthand()
                                        {
                                            changed_submodules.push((
                                                submodule_name,
                                                branch_name.to_string(),
                                            ));
                                        }
                                    },
                                    std::result::Result::Err(_) => continue,
                                }
                            },
                            std::result::Result::Err(_) => continue,
                        }
                    }
                },
                std::result::Result::Err(_) => continue,
            }
        }
    }

    // Build the commit message
    let mut message = String::from("Switch and lock submodule state");

    if !changed_submodules.is_empty() {
        message.push_str("\n\nSwitched submodules:");
        for (name, branch) in &changed_submodules {
            message.push_str(&format!("\n- {}: {}", name, branch));
        }
    }

    std::result::Result::Ok((message, changed_submodules))
}
