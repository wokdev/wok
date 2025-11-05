use anyhow::*;
use serde::{Deserialize, Serialize};
use std::{fs, path};

const CONFIG_CURRENT_VERSION: &str = "1.0";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub path: path::PathBuf,
    pub head: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skip_for: Vec<String>,
}

/// Config schema for `wok.toml`
///
/// A repository containing `wok.toml` file serves as an "umbrella" repo for a
/// workspace containing several repos.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: String,
    #[serde(rename = "repo")]
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
            skip_for: vec![],
        });
        true
    }

    pub fn remove_repo(&mut self, path: &path::Path) -> bool {
        assert!(!path.is_absolute());

        let mut removed = false;
        self.repos.retain(|r| {
            if r.path != path {
                return true;
            }
            removed = true;
            false
        });
        removed
    }

    pub fn set_repo_head(&mut self, path: &path::Path, head: &String) -> bool {
        assert!(!path.is_absolute());

        let mut updated = false;
        self.repos = self
            .repos
            .iter()
            .map(|r| -> Repo {
                let mut r = r.to_owned();
                if r.path != path {
                    return r;
                }
                if r.head == *head {
                    return r;
                }
                r.head = head.to_owned();
                updated = true;
                r
            })
            .collect();
        updated
    }

    /// Loads the workspace config from a file at the `config_path`.
    pub fn load(config_path: &path::Path) -> Result<Config> {
        let mut config: Config = toml::from_str(&Self::read(config_path)?)
            .context("Cannot parse the wok file")?;
        
        // Migrate from 1.0-experimental to 1.0
        if config.version == "1.0-experimental" {
            config.version = String::from("1.0");
            config.save(config_path)
                .context("Cannot save migrated wok file")?;
        }
        
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
        assert!(!path.is_absolute());
        self.repos.iter().any(|r| r.path == path)
    }
}

impl Repo {
    pub fn is_skipped_for(&self, command: &str) -> bool {
        self.skip_for
            .iter()
            .any(|skip| skip.eq_ignore_ascii_case(command))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}
