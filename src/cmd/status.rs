use anyhow::*;
use std::io::Write;

use crate::{config, repo};

pub fn status<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
    fetch: bool,
) -> Result<()> {
    // Fetch from remotes if requested
    if fetch {
        umbrella.fetch()?;
        for config_repo in &wok_config.repos {
            if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
                subrepo.fetch()?;
            }
        }
    }

    // Check if umbrella repo is clean
    let umbrella_clean = is_repo_clean(&umbrella.git_repo, Some(&wok_config.repos))?;
    let umbrella_emoji = if umbrella_clean { "✓" } else { "✗" };
    let clean_status = if umbrella_clean { "all clean" } else { "dirty" };

    // Get remote status for umbrella
    let remote_status = get_remote_status_string(umbrella, &umbrella.head)?;

    writeln!(
        stdout,
        "{} (umbrella) on branch '{}', {}{}",
        umbrella_emoji, &umbrella.head, clean_status, remote_status
    )?;

    // Show status for each configured subrepo
    for config_repo in &wok_config.repos {
        if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
            let subrepo_clean = is_repo_clean(&subrepo.git_repo, None)?;
            let subrepo_emoji = if subrepo_clean { "✓" } else { "✗" };
            let subrepo_clean_status =
                if subrepo_clean { "all clean" } else { "dirty" };
            let subrepo_remote_status =
                get_remote_status_string(subrepo, &subrepo.head)?;

            writeln!(
                stdout,
                "{} '{}' on branch '{}', {}{}",
                subrepo_emoji,
                config_repo.path.display(),
                &subrepo.head,
                subrepo_clean_status,
                subrepo_remote_status
            )?;
        }
    }

    Ok(())
}

fn get_remote_status_string(
    repo_obj: &repo::Repo,
    branch_name: &str,
) -> Result<String> {
    let tracking = match repo_obj.tracking_branch(branch_name)? {
        Some(tracking) => tracking,
        None => return Ok(String::new()), // No tracking branch, no status to show
    };

    match repo_obj.get_remote_comparison(branch_name)? {
        Some(repo::RemoteComparison::UpToDate) => Ok(format!(
            ", up to date with '{}'",
            tracking.remote_ref.replace("refs/remotes/", "")
        )),
        Some(repo::RemoteComparison::Ahead(count)) => {
            let commits = if count == 1 { "commit" } else { "commits" };
            Ok(format!(
                ", ahead of '{}' by {} {}",
                tracking.remote_ref.replace("refs/remotes/", ""),
                count,
                commits
            ))
        },
        Some(repo::RemoteComparison::Behind(count)) => {
            let commits = if count == 1 { "commit" } else { "commits" };
            Ok(format!(
                ", behind '{}' by {} {}",
                tracking.remote_ref.replace("refs/remotes/", ""),
                count,
                commits
            ))
        },
        Some(repo::RemoteComparison::Diverged(ahead, behind)) => Ok(format!(
            ", diverged from '{}' ({} ahead, {} behind)",
            tracking.remote_ref.replace("refs/remotes/", ""),
            ahead,
            behind
        )),
        Some(repo::RemoteComparison::NoRemote) => {
            Ok(String::new()) // Remote branch doesn't exist, don't show anything
        },
        None => Ok(String::new()), // No tracking branch
    }
}

fn is_repo_clean(
    git_repo: &git2::Repository,
    config_repos: Option<&[crate::config::Repo]>,
) -> Result<bool> {
    // Check if there are any uncommitted changes
    let mut status_options = git2::StatusOptions::new();
    status_options.include_ignored(false);
    status_options.include_untracked(true);

    let statuses = git_repo.statuses(Some(&mut status_options))?;

    // Check if repo is clean - ignore certain files that are expected
    for entry in statuses.iter() {
        let status = entry.status();
        let path = entry.path();

        // If it's just an untracked wok.toml file, we can consider the repo clean
        if status == git2::Status::WT_NEW && path == Some("wok.toml") {
            continue;
        }

        // If it's a newly added .gitmodules file, we can consider the repo clean
        if status == git2::Status::INDEX_NEW && path == Some(".gitmodules") {
            continue;
        }

        // If it's a newly added submodule directory, we can consider the repo clean
        if status == git2::Status::INDEX_NEW
            && let Some(path_str) = path
            && let Some(config_repos) = config_repos
            && config_repos
                .iter()
                .any(|r| r.path.to_string_lossy() == path_str)
        {
            continue;
        }

        // Any other status means the repo is not clean
        return Ok(false);
    }

    Ok(true)
}
