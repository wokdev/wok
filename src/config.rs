use anyhow::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path};

const CONFIG_CURRENT_VERSION: &str = "1.0-experimental";

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub head: String,
}

/// Config schema for `wok.toml`
///
/// A repository containing `wok.toml` file serves as an "umbrella" repo for a
/// workspace containing several repos.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: String,
    pub repos: BTreeMap<path::PathBuf, Repo>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            version: String::from(CONFIG_CURRENT_VERSION),
            repos: BTreeMap::new(),
        }
    }

    pub fn add_repo(&mut self, subrepo_path: &path::Path, head: &str) -> bool {
        assert!(!subrepo_path.is_absolute());

        if self.has_repo_path(subrepo_path) {
            return false;
        }

        self.repos.insert(
            path::PathBuf::from(subrepo_path),
            Repo {
                head: String::from(head),
            },
        );
        true
    }

    pub fn remove_repo(&mut self, subrepo_path: &path::PathBuf) -> bool {
        assert!(!subrepo_path.is_absolute());

        self.repos.remove(subrepo_path).is_some()
    }

    /// Loads the workspace config from a file at the `config_path`.
    pub fn load(config_path: &path::Path) -> Result<Config> {
        let config = toml::from_str(&Self::read(config_path)?)
            .context("Cannot parse the wok file")?;
        Ok(config)
    }

    /// Reads the config file into a string (useful mainly for testing).
    pub fn read(config_path: &path::Path) -> Result<String> {
        fs::read_to_string(config_path).context("Cannot read the wok file")
    }

    /// Saves the workspace config to a file.
    pub fn save(&self, config_path: &path::Path) -> Result<()> {
        fs::write(config_path, self.dump()?).context("Cannot save the wok file")?;
        Ok(())
    }

    /// Returns config as TOML string (useful mainly for testing).
    pub fn dump(&self) -> Result<String> {
        Ok(toml::to_string(self).context("Cannot serialize config")?)
    }

    fn has_repo_path(&self, path: &path::Path) -> bool {
        self.repos.contains_key(path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}
