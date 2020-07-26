use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

mod error;

pub use error::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Repo {
    url: String,
    path: PathBuf,
    #[serde(rename = "ref", default = "Repo::default_ref")]
    ref_: String,
}

impl Repo {
    fn default_ref() -> String {
        "master".to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    version: String,
    #[serde(rename = "ref")]
    ref_: String,
    repos: Vec<Repo>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Config, serde_yaml::Error> {
        Ok(serde_yaml::from_str(&fs::read_to_string(path).unwrap())?)
    }

    pub fn save(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }
}
