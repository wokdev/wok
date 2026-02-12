use anyhow::*;
use std::io::Write;

use crate::{config, repo};

#[derive(Debug)]
struct RepoStatusRow {
    name: String,
    branch: String,
    status: RepoLineStatus,
}

#[derive(Debug)]
enum RepoLineStatus {
    Clean,
    NeedsLocking,
    NewCommits,
    HasUncommittedChanges,
}

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

    let mut status_rows: Vec<RepoStatusRow> =
        Vec::with_capacity(wok_config.repos.len() + 1);
    let mut umbrella_status =
        classify_umbrella_status(&umbrella.git_repo, &wok_config.repos)?;
    if matches!(umbrella_status, RepoLineStatus::Clean)
        && let Some(remote_comparison) =
            umbrella.get_remote_comparison(&umbrella.head)?
    {
        let has_local_new_commits = match remote_comparison {
            repo::RemoteComparison::Ahead(_) => true,
            repo::RemoteComparison::Diverged(ahead, _) => ahead > 0,
            repo::RemoteComparison::UpToDate
            | repo::RemoteComparison::Behind(_)
            | repo::RemoteComparison::NoRemote => false,
        };
        if has_local_new_commits {
            umbrella_status = RepoLineStatus::NewCommits;
        }
    }
    status_rows.push(RepoStatusRow {
        name: "umbrella".to_string(),
        branch: umbrella.head.clone(),
        status: umbrella_status,
    });

    for config_repo in &wok_config.repos {
        if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
            let is_clean = is_repo_clean(&subrepo.git_repo, None)?;
            let has_new_commits =
                has_new_commits_vs_pointer(umbrella, &config_repo.path, subrepo)?;
            status_rows.push(RepoStatusRow {
                name: config_repo.path.display().to_string(),
                branch: subrepo.head.clone(),
                status: line_status_for_repo(is_clean, has_new_commits),
            });
        }
    }

    for row in &status_rows {
        render_status_row(stdout, row)?;
    }

    Ok(())
}

fn line_status_for_repo(is_clean: bool, has_new_commits: bool) -> RepoLineStatus {
    if !is_clean {
        RepoLineStatus::HasUncommittedChanges
    } else if has_new_commits {
        RepoLineStatus::NewCommits
    } else {
        RepoLineStatus::Clean
    }
}

fn render_status_row<W: Write>(stdout: &mut W, row: &RepoStatusRow) -> Result<()> {
    let (symbol, label) = match row.status {
        RepoLineStatus::Clean => ("✅", "clean"),
        RepoLineStatus::NeedsLocking => ("🔒", "needs locking"),
        RepoLineStatus::NewCommits => ("⬆", "new commits"),
        RepoLineStatus::HasUncommittedChanges => ("❌", "has uncommitted changes"),
    };
    writeln!(
        stdout,
        "{} {} [{}]: {}",
        symbol, row.name, row.branch, label
    )?;
    Ok(())
}

fn has_new_commits_vs_pointer(
    umbrella: &repo::Repo,
    subrepo_path: &std::path::Path,
    subrepo: &repo::Repo,
) -> Result<bool> {
    let submodule = match subrepo_path.to_str() {
        Some(path_str) => umbrella.git_repo.find_submodule(path_str)?,
        None => return Ok(false),
    };

    let pointer_oid = submodule.index_id().or_else(|| submodule.head_id());
    let Some(pointer_oid) = pointer_oid else {
        return Ok(false);
    };

    let subrepo_head_oid = subrepo.git_repo.head()?.peel_to_commit()?.id();
    Ok(pointer_oid != subrepo_head_oid)
}

fn classify_umbrella_status(
    git_repo: &git2::Repository,
    config_repos: &[crate::config::Repo],
) -> Result<RepoLineStatus> {
    let relevant_entries =
        collect_relevant_status_entries(git_repo, Some(config_repos))?;
    if relevant_entries.is_empty() {
        return Ok(RepoLineStatus::Clean);
    }

    let only_submodule_paths = relevant_entries.iter().all(|(_, path)| {
        path.as_ref().is_some_and(|path_str| {
            config_repos.iter().any(|repo_cfg| {
                repo_cfg.path.to_string_lossy().as_ref() == path_str.as_str()
            })
        })
    });

    if only_submodule_paths {
        Ok(RepoLineStatus::NeedsLocking)
    } else {
        Ok(RepoLineStatus::HasUncommittedChanges)
    }
}

fn is_repo_clean(
    git_repo: &git2::Repository,
    config_repos: Option<&[crate::config::Repo]>,
) -> Result<bool> {
    Ok(collect_relevant_status_entries(git_repo, config_repos)?.is_empty())
}

fn collect_relevant_status_entries(
    git_repo: &git2::Repository,
    config_repos: Option<&[crate::config::Repo]>,
) -> Result<Vec<(git2::Status, Option<String>)>> {
    // Check if there are any uncommitted changes
    let mut status_options = git2::StatusOptions::new();
    status_options.include_ignored(false);
    status_options.include_untracked(true);

    let statuses = git_repo.statuses(Some(&mut status_options))?;
    let mut relevant_entries = Vec::new();

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

        relevant_entries.push((status, path.map(str::to_string)));
    }

    Ok(relevant_entries)
}
