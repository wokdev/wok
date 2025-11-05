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
    let mut updated_repos = Vec::new(); // Track updated repos

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

            // Track updated repos
            if updated {
                let commit_hash = get_current_commit_hash(&subrepo.git_repo)?;
                updated_repos.push((
                    config_repo.path.to_string_lossy().to_string(),
                    config_repo.head.clone(),
                    commit_hash,
                ));
            }
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

    commit_submodule_updates(&umbrella.git_repo, &updated_repos)?;

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

fn commit_submodule_updates(
    git_repo: &git2::Repository,
    updated_repos: &[(String, String, String)], // (name, branch, commit_hash)
) -> Result<()> {
    let signature = git_repo.signature()?;
    let tree_id = git_repo.index()?.write_tree()?;
    let tree = git_repo.find_tree(tree_id)?;

    let head_ref = git_repo.head()?;
    let parent_commit = head_ref.peel_to_commit()?;
    let parent_tree = parent_commit.tree()?;

    // Build commit message with update details
    let commit_message =
        build_update_commit_message(git_repo, &parent_tree, &tree, updated_repos)?;

    git_repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(())
}

/// Build a commit message for update operation showing which repos were updated.
fn build_update_commit_message(
    git_repo: &git2::Repository,
    parent_tree: &git2::Tree,
    index_tree: &git2::Tree,
    updated_repos: &[(String, String, String)], // (name, branch, commit_hash)
) -> Result<String> {
    // Get diff between parent tree and staged index
    let diff = git_repo.diff_tree_to_tree(Some(parent_tree), Some(index_tree), None)?;

    let mut changed_submodules = Vec::new();

    // Build a map of updated repos for quick lookup
    let updated_map: std::collections::HashMap<_, _> = updated_repos
        .iter()
        .map(|(name, branch, hash)| (name.clone(), (branch.clone(), hash.clone())))
        .collect();

    // Iterate through changed files in the diff
    for delta in diff.deltas() {
        if let Some(file_path) = delta.new_file().path()
            && let Some(file_path_str) = file_path.to_str()
        {
            match git_repo.find_submodule(file_path_str) {
                std::result::Result::Ok(submodule) => {
                    let submodule_name = submodule.path().to_string_lossy().to_string();

                    // Check if this was one of the updated repos
                    if let Some((branch, hash)) = updated_map.get(&submodule_name) {
                        let short_hash = &hash[..std::cmp::min(8, hash.len())];
                        changed_submodules.push((
                            submodule_name,
                            format!("{} to {}", branch, short_hash),
                        ));
                    }
                },
                std::result::Result::Err(_) => continue,
            }
        }
    }

    // Build the commit message
    let mut message = String::from("Update submodules to latest");

    if !changed_submodules.is_empty() {
        message.push_str("\n\nUpdated submodules:");
        for (name, info) in &changed_submodules {
            message.push_str(&format!("\n- {}: {}", name, info));
        }
    }

    std::result::Result::Ok(message)
}
