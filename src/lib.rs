//! # Git Wok
//!
//! Git Wok lib is the library behind the [`git-wok`](https://github.com/lig/wok) tool.
//!
//! The `git-wok` binary allows to control several git repositories as a single
//! project.
//!
//! See [Git Wok's README](https://github.com/lig/wok/blob/master/README.md) and `git-wok --help` for details on how to use the tool.

pub mod cmd;
pub mod config;
pub mod repo;

pub const DEFAULT_CONFIG_NAME: &str = "wok.toml";
