use anyhow::{bail, Context, Result};
use clap::Parser;
use std::{env, path};

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
    /// setting their branch to the `main-branch`.
    Init {
        /// Switch all submodules to the branch matching umbrella's head branch.
        #[clap(long, action)]
        sync: bool,
    },
    // #[clap(flatten)]
    // CommandConfigured(CommandConfigured),
}

#[derive(Debug, Parser)]
enum CommandConfigured {
    /// Adds a new project to the workspace.
    /// Note: An existing submodule could be imported using `wok import`
    /// command.
    Add {
        /// Git repo url.
        #[clap()]
        git_url: String,

        /// Path inside the umbrella repo to create submodule at.
        #[clap(parse(from_os_str))]
        module_path: path::PathBuf,
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

    match args.cmd {
        Command::Init { sync } => {
            if wok_file_path.exists() {
                bail!("Wok file already exists at `{}`", wok_file_path.display());
            };
            wok::cmd::init(&wok_file_path, sync)?
        },
        // Command::CommandConfigured(cmd_configured) => {
        //     if !wok_file_path.exists() {
        //         bail!("Wok file not found at `{}`", wok_file_path.display());
        //     };
        //     let mut state = wok::State::new(&wok_file_path)?;
        //     match cmd_configured {
        //         CommandConfigured::Add {
        //             git_url,
        //             module_path,
        //         } => {
        //             wok::cmd::add(&mut state, git_url, module_path)?;
        //         },
        //     }
        //     state.into_config()
        // },
    };

    Ok(())
}
