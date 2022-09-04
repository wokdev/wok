use anyhow::*;
use std::{env, path};

use crate::{config, repo};

pub fn init(config_path: &path::Path, umbrella: &repo::Repo, sync: bool) -> Result<()> {
    let mut wok_config: config::Config = Default::default();

    for repo in umbrella.subrepos.iter() {
        let repo_path = repo.work_dir.strip_prefix(&umbrella.work_dir)?;

        wok_config.add_repo(repo_path, &repo.head);

        if sync {
            repo.sync()?;
            println!(
                "Switched repo at `{}` to the tip of the `{}` branch",
                repo_path.display(),
                repo.head
            );
        }
    }

    wok_config.save(config_path)?;
    println!(
        "Created config at `{}`",
        config_path
            .strip_prefix(env::current_dir()?)
            .unwrap_or(config_path)
            .display()
    );
    Ok(())
}
