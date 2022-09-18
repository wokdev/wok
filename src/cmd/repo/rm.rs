use anyhow::*;
use std::path;

use crate::config;

pub fn rm(
    wok_config: &mut config::Config,
    submodule_path: &path::PathBuf,
) -> Result<bool> {
    if !wok_config.remove_repo(submodule_path) {
        println!("No subrepo at `{}` in config", submodule_path.display());
        return Ok(false);
    }

    println!("Removed subrepo at `{}`", submodule_path.display());
    Ok(true)
}
