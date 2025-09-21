use anyhow::{bail, Context, Result};
use clap::Parser;
use std::{env, io::stdout, path};
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
    wokfile_path: path::PathBuf,

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

    /// Lock submodule state by committing current submodule commits
    Lock,
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

    let wokfile_path = {
        let wokfile_path = args.wokfile_path;
        if wokfile_path.is_absolute() {
            wokfile_path
        } else {
            env::current_dir()
                .context("Cannot access the current directory")?
                .join(wokfile_path)
        }
    };

    let umbrella = wok::repo::Repo::new(
        wokfile_path.parent().with_context(|| {
            format!("Cannot open work dir for `{}`", wokfile_path.display())
        })?,
        None,
    )?;

    let mut output = stdout();

    match args.cmd {
        Command::Init {} => {
            if wokfile_path.exists() {
                bail!("Wok file already exists at `{}`", wokfile_path.display());
            };

            wok::cmd::init(&wokfile_path, &umbrella, &mut output)?
        },
        Command::App(app_cmd) => {
            if !wokfile_path.exists() {
                bail!("Wok file not found at `{}`", wokfile_path.display());
            };

            let mut wok_config = wok::config::Config::load(&wokfile_path)?;

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
                App::Lock => {
                    wok::cmd::lock(&mut wok_config, &umbrella, &mut output)?;
                    false // Don't save config for lock command
                },
            } {
                wok_config.save(&wokfile_path)?;
            }
        },
    };

    Ok(())
}
