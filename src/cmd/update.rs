use anyhow::*;
use std::io::Write;

use crate::{config, repo};

pub fn update<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
    no_commit: bool,
    include_umbrella: bool,
) -> Result<()> {
    writeln!(stdout, "Updating repositories...")?;

    let mut saw_subrepo_updates = false;
    let mut saw_conflicts = false;

    if include_umbrella {
        let (_, conflicts) = update_repo(umbrella, &umbrella.head, "umbrella", stdout)?;
        saw_conflicts |= conflicts;
    }

    // Step 1: Update each repo with fetch and merge
    for config_repo in &wok_config.repos {
        if config_repo.is_skipped_for("update") {
            continue;
        }

        if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
            let label = config_repo.path.display().to_string();
            let (updated, conflicts) =
                update_repo(subrepo, &config_repo.head, &label, stdout)?;
            saw_subrepo_updates |= updated;
            saw_conflicts |= conflicts;
        }
    }

    // Step 2: Stage all submodule changes in umbrella repo
    let staged_changes = stage_submodule_changes(&umbrella.git_repo)?;

    if saw_conflicts {
        writeln!(
            stdout,
            "Skipped committing umbrella repo due to merge conflicts"
        )?;
        return Ok(());
    }

    if no_commit {
        if staged_changes || saw_subrepo_updates {
            writeln!(
                stdout,
                "Changes staged; commit skipped because --no-commit was provided"
            )?;
        } else {
            writeln!(stdout, "No submodule updates detected; nothing to commit")?;
        }
        return Ok(());
    }

    // Step 3: Commit the updated submodule state
    if !staged_changes {
        writeln!(stdout, "No submodule updates detected; nothing to commit")?;
        return Ok(());
    }

    commit_submodule_updates(&umbrella.git_repo)?;

    writeln!(stdout, "Updated submodule state committed")?;
    Ok(())
}

fn update_repo<W: Write>(
    repo: &repo::Repo,
    branch_name: &str,
    label: &str,
    stdout: &mut W,
) -> Result<(bool, bool)> {
    // Switch to the desired branch first
    repo.switch(branch_name)?;

    // Attempt to merge with remote changes
    let merge_result = repo.merge(branch_name)?;

    // Get the current commit hash for reporting
    let current_commit = get_current_commit_hash(&repo.git_repo)?;
    let short_commit = &current_commit[..std::cmp::min(8, current_commit.len())];

    let mut updated = false;
    let mut conflicts = false;

    match merge_result {
        repo::MergeResult::UpToDate => {
            writeln!(
                stdout,
                "- '{}': already up to date on '{}' ({})",
                label, branch_name, short_commit
            )?;
        },
        repo::MergeResult::FastForward => {
            updated = true;
            writeln!(
                stdout,
                "- '{}': fast-forwarded '{}' to {}",
                label, branch_name, short_commit
            )?;
        },
        repo::MergeResult::Merged => {
            updated = true;
            writeln!(
                stdout,
                "- '{}': merged '{}' to {}",
                label, branch_name, short_commit
            )?;
        },
        repo::MergeResult::Rebased => {
            updated = true;
            writeln!(
                stdout,
                "- '{}': rebased '{}' to {}",
                label, branch_name, short_commit
            )?;
        },
        repo::MergeResult::Conflicts => {
            conflicts = true;
            writeln!(
                stdout,
                "- '{}': merge conflicts in '{}' ({}), manual resolution required",
                label, branch_name, short_commit
            )?;
        },
    }

    Ok((updated, conflicts))
}

fn get_current_commit_hash(git_repo: &git2::Repository) -> Result<String> {
    let head = git_repo.head()?;
    let commit = head.peel_to_commit()?;
    Ok(commit.id().to_string())
}

fn stage_submodule_changes(git_repo: &git2::Repository) -> Result<bool> {
    let head_tree = git_repo
        .head()
        .ok()
        .and_then(|head| head.peel_to_tree().ok());
    let mut index = git_repo.index()?;

    for submodule in git_repo.submodules()? {
        let submodule_path = submodule.path();

        // Only stage submodules that have a head (are initialized)
        if let Some(_submodule_oid) = submodule.head_id() {
            index.add_path(submodule_path)?;
        }
    }

    index.write()?;

    if let Some(tree) = head_tree.as_ref() {
        let diff = git_repo.diff_tree_to_index(Some(tree), Some(&index), None)?;
        Ok(diff.deltas().len() > 0)
    } else {
        Ok(!index.is_empty())
    }
}

fn commit_submodule_updates(git_repo: &git2::Repository) -> Result<()> {
    let commit_message = "Update submodules to latest";
    let signature = git_repo.signature()?;
    let tree_id = git_repo.index()?.write_tree()?;
    let tree = git_repo.find_tree(tree_id)?;

    let head_ref = git_repo.head()?;
    let parent_commit = head_ref.peel_to_commit()?;

    git_repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(())
}
