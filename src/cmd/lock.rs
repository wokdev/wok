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

    // Commit the submodule state
    let commit_message = "Lock submodule state";
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

    writeln!(stdout, "Locked submodule state")?;
    Ok(())
}
