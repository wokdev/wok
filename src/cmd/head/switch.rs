use anyhow::*;

use crate::{config, repo};

pub fn switch(wok_config: &mut config::Config, umbrella: &repo::Repo) -> Result<bool> {
    let mut config_updated = false;

    for subrepo in umbrella.subrepos.iter() {
        subrepo.switch(&umbrella.head)?;
        config_updated |= wok_config.set_repo_head(
            subrepo.work_dir.strip_prefix(&umbrella.work_dir)?,
            &umbrella.head,
        );
    }

    Ok(config_updated)
}
