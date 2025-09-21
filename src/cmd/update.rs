use anyhow::*;
use std::io::Write;

use crate::{config, repo};

pub fn update<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
) -> Result<()> {
    writeln!(stdout, "Updating submodules...")?;

    // Step 1: Switch each repo to its configured branch
    for config_repo in &wok_config.repos {
        if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
            // Switch to configured branch
            subrepo.switch(&config_repo.head)?;

            // Get the current commit hash for reporting
            let current_commit = get_current_commit_hash(&subrepo.git_repo)?;

            writeln!(
                stdout,
                "- '{}': switched to '{}', updated to {}",
                config_repo.path.display(),
                config_repo.head,
                &current_commit[..8]
            )?;
        }
    }

    // Step 2: Stage all submodule changes in umbrella repo
    stage_submodule_changes(&umbrella.git_repo)?;

    // Step 3: Commit the updated submodule state
    commit_submodule_updates(&umbrella.git_repo)?;

    writeln!(stdout, "Updated submodule state committed")?;
    Ok(())
}

fn get_current_commit_hash(git_repo: &git2::Repository) -> Result<String> {
    let head = git_repo.head()?;
    let commit = head.peel_to_commit()?;
    Ok(commit.id().to_string())
}

fn stage_submodule_changes(git_repo: &git2::Repository) -> Result<()> {
    let mut index = git_repo.index()?;

    for submodule in git_repo.submodules()? {
        let submodule_path = submodule.path();

        // Only stage submodules that have a head (are initialized)
        if let Some(_submodule_oid) = submodule.head_id() {
            index.add_path(submodule_path)?;
        }
    }

    index.write()?;
    Ok(())
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
