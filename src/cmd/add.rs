use anyhow::*;
use std::path;

use crate::{config, repo};

pub fn add(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    submodule_path: &path::PathBuf,
) -> Result<bool> {
    let subrepo_path = &umbrella.work_dir.join(submodule_path);
    let subrepo = umbrella
        .get_subrepo_by_path(subrepo_path)
        .with_context(|| {
            format!("Cannot find submodule at `{}`", subrepo_path.display())
        })?;

    if !wok_config.add_repo(
        subrepo.work_dir.strip_prefix(&umbrella.work_dir)?,
        &subrepo.head,
    ) {
        println!(
            "Not adding existing subrepo at `{}`",
            &subrepo.work_dir.display()
        );
        return Ok(false);
    }

    println!(
        "Added subrepo at `{}` with head `{}`",
        &subrepo.work_dir.display(),
        &subrepo.head
    );
    Ok(true)
}
