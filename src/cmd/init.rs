use anyhow::*;
use std::{env, path};

use crate::{config, repo};

pub fn init(config_path: &path::Path, sync: bool) -> Result<()> {
    let mut config = config::Config::new();

    let umbrella = repo::Repo::new(
        config_path.parent().with_context(|| {
            format!("Cannot open work dir for `{}`", config_path.display())
        })?,
        None,
    )?;

    for repo in umbrella.subrepos.iter() {
        let repo_path = repo.work_dir.strip_prefix(&umbrella.work_dir)?;

        config.add_repo(repo_path, &repo.head);

        if sync {
            repo.sync()?;
            println!(
                "Switched repo at `{}` to the tip of the `{}` branch",
                repo_path.display(),
                repo.head
            );
        }
    }

    config.save(config_path)?;
    println!(
        "Created config at `{}`",
        config_path
            .strip_prefix(env::current_dir()?)
            .unwrap_or(config_path)
            .display()
    );
    Ok(())
}
