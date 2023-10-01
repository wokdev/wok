use anyhow::*;
use std::io::Write;
use std::{env, path};

use crate::{config, repo};

pub fn init<W: Write>(
    config_path: &path::Path,
    umbrella: &repo::Repo,
    stdout: &mut W,
) -> Result<()> {
    let mut wok_config: config::Config = Default::default();

    for repo in umbrella.subrepos.iter() {
        let repo_path = repo.work_dir.strip_prefix(&umbrella.work_dir)?;

        wok_config.add_repo(repo_path, &repo.head);
    }

    wok_config.save(config_path)?;
    writeln!(
        stdout,
        "Created config at `{}`",
        config_path
            .strip_prefix(env::current_dir()?)
            .unwrap_or(config_path)
            .display()
    )?;
    Ok(())
}
