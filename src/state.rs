use git2;
use std::{fmt, path};

pub struct State {
    pub config: crate::Config,
    pub umbrella: git2::Repository,
    pub projects: Vec<git2::Repository>,
}

impl State {
    pub fn new(config_path: &path::Path) -> Result<Self, crate::Error> {
        let config =
            crate::Config::load(config_path).map_err(|e| crate::Error::from(&e))?;
        let umbrella = git2::Repository::open(&config_path.parent().unwrap())
            .map_err(|e| crate::Error::from(&e))?;
        let projects = config
            .repos
            .iter()
            .map(|repo| git2::Repository::open(&repo.path).unwrap())
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
