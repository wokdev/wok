use anyhow::*;
use serde::{Deserialize, Serialize};
use std::{fs, path};

const CONFIG_CURRENT_VERSION: &str = "1.0";

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub path: path::PathBuf,
    pub head: String,
}

/// Config schema for `wok.yaml`
///
/// A repository containing `wok.yaml` file serves as an "umbrella" repo for a
/// workspace containing several repos.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: String,
    pub repos: Vec<Repo>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            version: String::from(CONFIG_CURRENT_VERSION),
            repos: vec![],
        }
    }

    pub fn add_repo(&mut self, path: &path::Path, head: &str) -> bool {
        assert!(!path.is_absolute());

        if self.has_repo_path(path) {
            return false;
        }

        self.repos.push(Repo {
            path: path::PathBuf::from(path),
            head: String::from(head),
        });
        true
    }

    /// Loads the workspace config from a file at the `config_path`.
    pub fn load(config_path: &path::Path) -> Result<Config> {
        let config = serde_yaml::from_str(
            &fs::read_to_string(config_path).context("Cannot read the wok file")?,
        )
        .context("Cannot parse the wok file")?;
        Ok(config)
    }

    /// Saves the workspace config to a file.
    pub fn save(&self, config_path: &path::Path) -> Result<()> {
        fs::write(config_path, self.dump()?).context("Cannot save the wok file")?;
        Ok(())
    }

    /// Returns config as YAML string (useful mainly for testing).
    pub fn dump(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self).context("Cannot serialize config")?)
    }

    fn has_repo_path(&self, path: &path::Path) -> bool {
        self.repos.iter().any(|repo| repo.path == path)
    }
}
