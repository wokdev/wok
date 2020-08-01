use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub const CONFIG_CURRENT_VERSION: &str = "1.0";

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub url: Option<String>,
    pub path: PathBuf,
    #[serde(rename = "ref", default = "Repo::default_ref")]
    pub ref_: String,
}

impl Repo {
    fn default_ref() -> String {
        String::from("main")
    }
}

/// Config schema for `wok.yml`
///
/// A repository containing `wok.yml` file serves as an "umbrella" repo for a
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
    /// Loads the workspace config from a file at the `path`.
    pub fn load(path: &PathBuf) -> Result<Config, serde_yaml::Error> {
        let config = serde_yaml::from_str(&fs::read_to_string(path).unwrap())?;
        eprintln!("Wok config loaded from `{}`", path.to_string_lossy());
        Ok(config)
    }

    /// Saves the workspace config to a file.
    pub fn save(&self, path: &PathBuf) -> Result<(), serde_yaml::Error> {
        fs::write(path, self.dump()?).unwrap();
        eprintln!("Wok config saved to `{}`", path.to_string_lossy());
        Ok(())
    }

    /// Returns config as YAML string (useful mainly for testing).
    pub fn dump(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}
