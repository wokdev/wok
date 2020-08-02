use git2;
use std::path;

pub struct State {
    pub config_path: path::PathBuf,
    pub config: crate::Config,
    pub umbrella: git2::Repository,
    pub projects: Vec<git2::Repository>,
}
