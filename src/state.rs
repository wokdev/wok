use anyhow::Result;
use std::{fmt, path};

pub struct State {
    pub config: crate::Config,
    pub umbrella: git2::Repository,
    pub projects: Vec<git2::Repository>,
}

impl State {
    pub fn new(config_path: &path::Path) -> Result<Self> {
        let config = crate::Config::load(config_path)?;
        let umbrella = git2::Repository::open(&config_path.parent().unwrap())?;
        let projects = config
            .repos
            .iter()
            .map(|repo| {
                git2::Repository::open(umbrella.workdir().unwrap().join(&repo.path))
                    .unwrap()
            })
            .collect();
        Ok(Self {
            config,
            umbrella,
            projects,
        })
    }

    pub fn into_config(self) -> crate::Config {
        self.config
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let projects_debug: Vec<&path::Path> =
            self.projects.iter().map(|r| r.path()).collect();
        f.debug_struct("State")
            .field("umbrella", &self.umbrella.path())
            .field("projects", &projects_debug)
            .finish()
    }
}
