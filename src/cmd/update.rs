use anyhow::*;
use std::io::Write;

use crate::{config, repo};

pub fn update<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
    no_commit: bool,
) -> Result<()> {
    writeln!(stdout, "Updating submodules...")?;

    let mut saw_updates = false;
    let mut saw_conflicts = false;

    // Step 1: Update each repo with fetch and merge
    for config_repo in &wok_config.repos {
        if config_repo.is_skipped_for("update") {
            continue;
        }

        if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
            // Switch to configured branch first
            subrepo.switch(&config_repo.head)?;

            // Attempt to merge with remote changes
            let merge_result = subrepo.merge(&config_repo.head)?;

            // Get the current commit hash for reporting
            let current_commit = get_current_commit_hash(&subrepo.git_repo)?;

            // Report the result based on merge outcome
            match merge_result {
                repo::MergeResult::UpToDate => {
                    writeln!(
                        stdout,
                        "- '{}': already up to date on '{}' ({})",
                        config_repo.path.display(),
                        config_repo.head,
                        &current_commit[..8]
                    )?;
                },
                repo::MergeResult::FastForward => {
                    saw_updates = true;
                    writeln!(
                        stdout,
                        "- '{}': fast-forwarded '{}' to {}",
                        config_repo.path.display(),
                        config_repo.head,
                        &current_commit[..8]
                    )?;
                },
                repo::MergeResult::Merged => {
                    saw_updates = true;
                    writeln!(
                        stdout,
                        "- '{}': merged '{}' to {}",
                        config_repo.path.display(),
                        config_repo.head,
                        &current_commit[..8]
                    )?;
                },
                repo::MergeResult::Conflicts => {
                    saw_conflicts = true;
                    writeln!(
                        stdout,
                        "- '{}': merge conflicts in '{}' ({}), manual resolution required",
                        config_repo.path.display(),
                        config_repo.head,
                        &current_commit[..8]
                    )?;
                },
            }
        }
    }

    // Step 2: Stage all submodule changes in umbrella repo
    let staged_changes = stage_submodule_changes(&umbrella.git_repo)?;

    if saw_conflicts {
        writeln!(
            stdout,
            "Skipped committing umbrella repo due to merge conflicts in subrepos"
        )?;
        return Ok(());
    }

    if no_commit {
        if staged_changes || saw_updates {
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
