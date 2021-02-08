use serde::{Deserialize, Serialize};
use std::{fs, path};

pub const CONFIG_CURRENT_VERSION: &str = "1.0";

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub url: Option<String>,
    pub path: path::PathBuf,
    #[serde(rename = "ref", default = "Repo::default_ref")]
    pub ref_: String,
}

impl Repo {
    fn default_ref() -> String {
        String::from("main")
    }
}

/// Config schema for `wok.yaml`
///
/// A repository containing `wok.yaml` file serves as an "umbrella" repo for a
/// workspace containing several repos.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: String,
    #[serde(rename = "ref")]
    pub ref_: String,
    pub repos: Vec<Repo>,
}

impl Config {
    /// Loads the workspace config from a file at the `config_path`.
    pub fn load(config_path: &path::Path) -> Result<Config, serde_yaml::Error> {
        let config = serde_yaml::from_str(&fs::read_to_string(config_path).unwrap())?;
        eprintln!("Wok config loaded from `{}`", config_path.to_string_lossy());
        Ok(config)
    }

    /// Saves the workspace config to a file.
    pub fn save(&self, config_path: &path::Path) -> Result<(), serde_yaml::Error> {
        fs::write(config_path, self.dump()?).unwrap();
        eprintln!("Wok config saved to `{}`", config_path.to_string_lossy());
        Ok(())
    }

    /// Returns config as YAML string (useful mainly for testing).
    pub fn dump(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}
