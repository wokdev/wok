use anyhow::*;
use std::io::Write;

use crate::{config, repo};

pub fn status<W: Write>(
    _wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
) -> Result<()> {
    // On branch '<>', all clean
    // - '<subrepo>' is on branch '<>', all clean
    writeln!(stdout, "On branch '{}'", &umbrella.head)?;
    Ok(())
}
