use clap::Clap;
use std::path::PathBuf;
use wok;

#[derive(Debug, Clap)]
#[clap(
    name = "wok",
    about = "Wok -- control several git repositories as a single project."
)]
struct Opts {
    /// Wok workspace file path. Defaults to `wok.yaml` in the umbrella repo
    /// root.
    #[clap(short('f'), long, parse(from_os_str))]
    wok_file: Option<PathBuf>,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Clap)]
enum Command {
    /// Inits the wok file in the workspace "umbrella" repo.
    /// Requires the git repo to be inited already.
    /// Introspects existing submodules and adds them to the workspace config
    /// setting their branch to the `main-branch`.
    Init {
        /// Sets the workspace main branch to `main-branch` if provided
        /// otherwise uses the currently active branch in the "umbrella"
        /// repo.
        #[clap(short, long)]
        main_branch: Option<String>,

        /// Disables submodule introspection.
        #[clap(short, long)]
        no_introspect: bool,
    },

    #[clap(flatten)]
    CommandConfigured(CommandConfigured),
}

#[derive(Debug, Clap)]
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
        module_path: PathBuf,
    },
}

fn main() -> Result<(), wok::Error> {
    let opt = Opts::parse();

    let wok_file = match opt.wok_file {
        Some(path) => path,
        None => git2::Repository::open_from_env()
            .map_err(|e| wok::Error::from(&e))?
            .workdir()
            .unwrap()
            .join("wok.yaml"),
    };

    let config = match opt.cmd {
        Command::Init {
            main_branch,
            no_introspect,
        } => {
            if wok_file.exists() {
                return Err(wok::Error::new(format!(
                    "Config already exists at `{}`",
                    wok_file.to_string_lossy()
                )));
            };
            wok::cmd::init(wok_file.parent().unwrap(), main_branch, no_introspect)?
        },
        Command::CommandConfigured(cmd_configured) => {
            if !wok_file.exists() {
                return Err(wok::Error::new(format!(
                    "Config not found at `{}`",
                    wok_file.to_string_lossy()
                )));
            };
            let mut state = wok::State::new(&wok_file)?;
            match cmd_configured {
                CommandConfigured::Add {
                    git_url,
                    module_path,
                } => {
                    wok::cmd::add(&mut state, git_url, module_path)?;
                },
            }
            state.into_config()
        },
    };

    config.save(&wok_file).map_err(|e| wok::Error::from(&e))?;

    assert!(wok_file.exists());

    let config = wok::Config::load(&wok_file).map_err(|e| wok::Error::from(&e))?;

    eprintln!("{:?}", config);

    Ok(())
}
