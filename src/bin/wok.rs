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

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
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

    match opt.cmd {
        Command::Init { .. } => {
            if wok_file.exists() {
                return Err(wok::Error::new(format!(
                    "Config already exists at `{}`",
                    wok_file.to_string_lossy()
                )));
            }
        },
        _ => {
            if !wok_file.exists() {
                return Err(wok::Error::new(format!(
                    "Config not found at `{}`",
                    wok_file.to_string_lossy()
                )));
            }
        },
    };

    let config = match opt.cmd {
        Command::Init {
            main_branch,
            no_introspect,
        } => wok::cmd::init(main_branch, no_introspect)?,
    };

    config.save(&wok_file).map_err(|e| wok::Error::from(&e))?;

    assert!(wok_file.exists());

    let config = wok::Config::load(&wok_file).map_err(|e| wok::Error::from(&e))?;

    eprintln!("{:?}", config);

    Ok(())
}
