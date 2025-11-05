use anyhow::*;
use std::io::Write;

use crate::{config, repo};

pub fn lock<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
) -> Result<()> {
    // Ensure each repo is switched to its configured branch
    for config_repo in &wok_config.repos {
        if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
            // Switch subrepo to its configured branch
            subrepo.switch(&config_repo.head)?;
        }
    }

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

    // Check if there are any changes to commit
    let signature = umbrella.git_repo.signature()?;
    let tree_id = umbrella.git_repo.index()?.write_tree()?;
    let tree = umbrella.git_repo.find_tree(tree_id)?;

    let head_ref = umbrella.git_repo.head()?;
    let parent_commit = head_ref.peel_to_commit()?;
    let parent_tree = parent_commit.tree()?;

    // If nothing changed, don't create a commit
    if tree.id() == parent_tree.id() {
        writeln!(stdout, "No submodule changes detected; nothing to lock")?;
        return Ok(());
    }

    // Build commit message with changed submodule summary
    let (commit_message, _changed_submodules) =
        build_lock_commit_message(umbrella, &parent_tree, &tree)?;

    umbrella.git_repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_message,
        &tree,
        &[&parent_commit],
    )?;

    writeln!(stdout, "Locked submodule state")?;
    Ok(())
}

/// Build a commit message for lock operation and return changed submodule info.
/// Returns (commit_message, changed_submodules_list)
fn build_lock_commit_message(
    umbrella: &repo::Repo,
    parent_tree: &git2::Tree,
    index_tree: &git2::Tree,
) -> Result<(String, Vec<(String, String)>)> {
    // Get diff between parent tree and staged index
    let diff = umbrella.git_repo.diff_tree_to_tree(
        Some(parent_tree),
        Some(index_tree),
        None,
    )?;

    let mut changed_submodules = Vec::new();

    // Iterate through changed files in the diff
    for delta in diff.deltas() {
        if let Some(file_path) = delta.new_file().path() {
            // Convert Path to str for find_submodule
            if let Some(file_path_str) = file_path.to_str() {
                // Check if this file is a submodule
                match umbrella.git_repo.find_submodule(file_path_str) {
                    std::result::Result::Ok(submodule) => {
                        // Get the submodule repository and its HEAD
                        let submodule_repo_path =
                            umbrella.work_dir.join(submodule.path());
                        match git2::Repository::open(&submodule_repo_path) {
                            std::result::Result::Ok(subrepo_git) => {
                                // Get the actual HEAD of the submodule (not from umbrella's index)
                                match subrepo_git.head() {
                                    std::result::Result::Ok(head_ref) => {
                                        match head_ref.peel_to_commit() {
                                            std::result::Result::Ok(commit) => {
                                                let message = commit
                                                    .message()
                                                    .unwrap_or("(no message)");

                                                // Get first line of commit message and truncate to 50 chars
                                                let first_line = message
                                                    .lines()
                                                    .next()
                                                    .unwrap_or("(no message)");
                                                let truncated = if first_line.len() > 50
                                                {
                                                    format!("{}...", &first_line[..47])
                                                } else {
                                                    first_line.to_string()
                                                };

                                                let submodule_name = submodule
                                                    .path()
                                                    .to_string_lossy()
                                                    .to_string();
                                                changed_submodules
                                                    .push((submodule_name, truncated));
                                            },
                                            std::result::Result::Err(_) => continue,
                                        }
                                    },
                                    std::result::Result::Err(_) => continue,
                                }
                            },
                            std::result::Result::Err(_) => continue,
                        }
                    },
                    std::result::Result::Err(_) => continue,
                }
            }
        }
    }

    // Build the commit message
    let mut message = String::from("Lock submodule state");

    if !changed_submodules.is_empty() {
        message.push_str("\n\nChanged submodules:");
        for (name, commit_msg) in &changed_submodules {
            message.push_str(&format!("\n- {}: {}", name, commit_msg));
        }
    }

    std::result::Result::Ok((message, changed_submodules))
}
