use anyhow::*;
use std::io::Write;
use std::result::Result::Ok;

use crate::{config, repo};

pub fn push<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
    set_upstream: bool,
    all: bool,
    branch_name: Option<&str>,
    target_repos: &[std::path::PathBuf],
) -> Result<()> {
    // Determine the target branch
    let target_branch = match branch_name {
        Some(name) => name.to_string(),
        None => umbrella.head.clone(),
    };

    // Determine which repos to push
    let repos_to_push = if all {
        // Push all configured repos
        wok_config.repos.clone()
    } else if !target_repos.is_empty() {
        // Push only specified repos
        wok_config
            .repos
            .iter()
            .filter(|config_repo| target_repos.contains(&config_repo.path))
            .cloned()
            .collect()
    } else {
        // Push repos that match the current main repo branch
        wok_config
            .repos
            .iter()
            .filter(|config_repo| config_repo.head == umbrella.head)
            .cloned()
            .collect()
    };

    if repos_to_push.is_empty() {
        writeln!(stdout, "No repositories to push")?;
        return Ok(());
    }

    writeln!(
        stdout,
        "Pushing {} repositories to branch '{}'...",
        repos_to_push.len(),
        target_branch
    )?;

    // Push each repo
    for config_repo in &repos_to_push {
        if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
            match push_repo(subrepo, &target_branch, set_upstream) {
                Ok(result) => match result {
                    PushResult::Pushed => {
                        writeln!(
                            stdout,
                            "- '{}': pushed to '{}'",
                            config_repo.path.display(),
                            target_branch
                        )?;
                    },
                    PushResult::UpstreamSet => {
                        writeln!(
                            stdout,
                            "- '{}': pushed to '{}' and set upstream",
                            config_repo.path.display(),
                            target_branch
                        )?;
                    },
                    PushResult::UpToDate => {
                        writeln!(
                            stdout,
                            "- '{}': already up to date",
                            config_repo.path.display()
                        )?;
                    },
                    PushResult::NoRemote => {
                        writeln!(
                            stdout,
                            "- '{}': no remote configured, skipping",
                            config_repo.path.display()
                        )?;
                    },
                },
                Err(e) => {
                    writeln!(
                        stdout,
                        "- '{}': failed to push to '{}' - {}",
                        config_repo.path.display(),
                        target_branch,
                        e
                    )?;
                },
            }
        }
    }

    writeln!(
        stdout,
        "Successfully processed {} repositories",
        repos_to_push.len()
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum PushResult {
    Pushed,
    UpstreamSet,
    UpToDate,
    NoRemote,
}

fn push_repo(
    repo: &repo::Repo,
    branch_name: &str,
    set_upstream: bool,
) -> Result<PushResult> {
    // Get the remote name for this branch
    let remote_name = repo.get_remote_name_for_branch(branch_name)?;

    // Check if remote exists
    let mut remote = match repo.git_repo.find_remote(&remote_name) {
        Ok(remote) => remote,
        Err(_) => {
            return Ok(PushResult::NoRemote);
        },
    };

    // Get the current branch reference
    let branch_ref = format!("refs/heads/{}", branch_name);

    // Check if the branch exists locally
    if repo.git_repo.refname_to_id(&branch_ref).is_err() {
        return Err(anyhow!("Branch '{}' does not exist locally", branch_name));
    }

    // Prepare the refspec for pushing
    let refspec = format!("{}:refs/heads/{}", branch_ref, branch_name);

    // Perform the push
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(git2::RemoteCallbacks::new());

    match remote.push(&[&refspec], Some(&mut push_options)) {
        Ok(_) => {
            if set_upstream {
                // Set the upstream branch
                set_upstream_branch(repo, branch_name, &remote_name)?;
                Ok(PushResult::UpstreamSet)
            } else {
                Ok(PushResult::Pushed)
            }
        },
        Err(e) => {
            // Check if it's an "up to date" error
            if e.message().contains("up to date")
                || e.message().contains("non-fast-forward")
            {
                Ok(PushResult::UpToDate)
            } else {
                Err(e.into())
            }
        },
    }
}

fn set_upstream_branch(
    repo: &repo::Repo,
    branch_name: &str,
    remote_name: &str,
) -> Result<()> {
    // Update the branch configuration to set upstream
    let mut config = repo.git_repo.config()?;
    config.set_str(&format!("branch.{}.remote", branch_name), remote_name)?;
    config.set_str(
        &format!("branch.{}.merge", branch_name),
        &format!("refs/heads/{}", branch_name),
    )?;

    Ok(())
}
