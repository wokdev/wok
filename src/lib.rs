//! # Wok
//!
//! Wok lib is the library behind the [`wok`](https://github.com/lig/wok) tool.
//!
//! The `wok` binary allows to control several git repositories as a single
//! project.
//!
//! See [Wok's README](https://github.com/lig/wok/blob/master/README.md) and `wok --help` for details on how to use the tool.

mod config;
mod error;

pub mod cmd;

pub use config::{Config, CONFIG_CURRENT_VERSION};
pub use error::Error;
