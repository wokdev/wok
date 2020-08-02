use std::path::PathBuf;
use structopt::StructOpt;
use wok;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "wok",
    about = "Wok -- control several git repositories as a single project."
)]
struct Opt {
    /// Wok workspace file path. Defaults to `wok.yml` in the umbrella repo
    /// root.
    #[structopt(short("f"), long, parse(from_os_str))]
    wok_file: Option<PathBuf>,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Inits the wok file in the workspace "umbrella" repo.
    /// Requires the git repo to be inited already.
    /// Introspects existing submodules and adds them to the workspace config
    /// setting their branch to the `main-branch`.
    Init {
        #[structopt(
            short,
            long,
            help(
                "Sets the workspace main branch to `main-branch` if provided \
                 otherwise uses the currently active branch in the \"umbrella\" repo."
            )
        )]
        main_branch: Option<String>,

        #[structopt(short, long, help("Disables submodule introspection."))]
        no_introspect: bool,
    },

    #[structopt(flatten)]
    CommandConfigured(CommandConfigured),
}

#[derive(Debug, StructOpt)]
enum CommandConfigured {
    /// Adds a new project to the workspace.
    /// Note: An existing submodule could be imported using `wok import` command
    Add {
        #[structopt(help("Git repo url."))]
        git_url: String,

        #[structopt(
            parse(from_os_str),
            help("Path inside the umbrella repo to create submodule at.")
        )]
        module_path: PathBuf,
    },
}

fn main() -> Result<(), wok::Error> {
    let opt = Opt::from_args();

    let wok_file = match opt.wok_file {
        Some(path) => path,
        None => git2::Repository::open_from_env()
            .map_err(|e| wok::Error::from(&e))?
            .workdir()
            .unwrap()
            .join("wok.yml"),
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
                    wok::cmd::add(&mut state, &git_url, &module_path)?;
                    state.into_config()
                },
            }
        },
    };

    config.save(&wok_file).map_err(|e| wok::Error::from(&e))?;

    assert!(wok_file.exists());

    let config = wok::Config::load(&wok_file).map_err(|e| wok::Error::from(&e))?;

    eprintln!("{:?}", config);

    Ok(())
}
