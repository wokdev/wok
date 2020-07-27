use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub url: String,
    pub path: PathBuf,
    #[serde(rename = "ref", default = "Repo::default_ref")]
    pub ref_: String,
}

impl Repo {
    fn default_ref() -> String {
        "master".to_string()
    }
}

/// Config schema for `wok.yml`
///
/// A repository containing `wok.yml` file serves as an "umbrella" repo for a workspace
/// containing several repos.
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
        Ok(serde_yaml::from_str(&fs::read_to_string(path).unwrap())?)
    }

    /// Saves the workspace config to its location.
    ///
    /// TODO: Implement the saving.
    pub fn save(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }
}
