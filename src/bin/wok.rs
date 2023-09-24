use anyhow::{bail, Context, Result};
use clap::Parser;
use std::{env, path};
use wok_dev as wok;

#[derive(Debug, Parser)]
#[clap(
    name = "wok",
    about = "Wok -- control several git repositories as a single project."
)]
struct Args {
    /// Wok file path.
    #[clap(
        global = true,
        short('f'),
        long,
        value_parser,
        default_value = wok::DEFAULT_CONFIG_NAME,
    )]
    wok_file_path: path::PathBuf,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Inits the wok file in the workspace "umbrella" repo.
    /// Requires the git repo to be inited already.
    /// Introspects existing submodules and adds them to the workspace config
    /// optionally switching them to the same branch.
    Init {},

    #[clap(flatten)]
    App(App),
}

#[derive(Debug, Parser)]
enum App {
    /// Change current subrepos' heads
    #[clap(subcommand)]
    Head(Head),

    /// Subrepos management
    #[clap(subcommand)]
    Repo(Repo),
}

#[derive(Debug, Parser)]
enum Head {
    /// Switches all subrepos' heads to the current umbrella's head branch.
    Switch,
}

#[derive(Debug, Parser)]
enum Repo {
    /// Adds an existing submodule to the wok workspace.
    Add {
        /// Path of the submodule relative to the umbrella repo.
        submodule_path: path::PathBuf,
    },
    /// Removes a submodule from the wok workspace.
    #[clap(name = "rm")]
    Remove {
        /// Path of the submodule relative to the umbrella repo.
        submodule_path: path::PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    let wok_file_path = {
        let wok_file_path = args.wok_file_path;
        if wok_file_path.is_absolute() {
            wok_file_path
        } else {
            env::current_dir()
                .context("Cannot access the current directory")?
                .join(wok_file_path)
        }
    };

    let umbrella = wok::repo::Repo::new(
        wok_file_path.parent().with_context(|| {
            format!("Cannot open work dir for `{}`", wok_file_path.display())
        })?,
        None,
    )?;

    match args.cmd {
        Command::Init {} => wok::cmd::init(&wok_file_path, &umbrella)?,
        Command::App(app_cmd) => {
            if !wok_file_path.exists() {
                bail!("Wok file not found at `{}`", wok_file_path.display());
            };

            let mut wok_config = wok::config::Config::load(&wok_file_path)?;

            if match app_cmd {
                App::Head(head_cmd) => match head_cmd {
                    Head::Switch => wok::cmd::head::switch(&mut wok_config, &umbrella)?,
                },
                App::Repo(repo_cmd) => match repo_cmd {
                    Repo::Add { submodule_path } => wok::cmd::repo::add(
                        &mut wok_config,
                        &umbrella,
                        &submodule_path,
                    )?,
                    Repo::Remove { submodule_path } => {
                        wok::cmd::repo::rm(&mut wok_config, &submodule_path)?
                    },
                },
            } {
                wok_config.save(&wok_file_path)?;
            }
        },
    };

    Ok(())
}
