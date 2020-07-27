use std::path::PathBuf;
use structopt::StructOpt;
use wok;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "wok",
    about = "Wok -- control several git repositories as a single project."
)]
struct Opt {
    /// Wok workspace file path
    #[structopt(default_value = "wok.yml", short("f"), long, parse(from_os_str))]
    wok_file: PathBuf,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Inits the wok file in the workspace "umbrella" repo.
    /// Requires the git repo to be inited already.
    /// Introspects existing submodules and adds them to the workspace config setting
    /// their branch to the `main-branch`.
    Init {
        #[structopt(
            short,
            long,
            help(
                "Sets the workspace main branch to `main-branch` if provided otherwise uses the currently active branch in the \"umbrella\" repo."
            )
        )]
        main_branch: Option<String>,

        #[structopt(short, long, help("Disables submodule introspection."))]
        no_introspect: bool,
    },
}

fn main() -> Result<(), wok::Error> {
    let opt = Opt::from_args();

    println!("{:?}", opt);

    return match opt.cmd {
        Command::Init {
            main_branch,
            no_introspect,
        } => wok::cmd::init(main_branch, no_introspect),
    };

    assert!(opt.wok_file.exists());

    let config = wok::Config::load(&opt.wok_file).map_err(|e| wok::Error::from(&e))?;

    println!("{:?}", config);
    println!("{:}", config.save());

    Ok(())
}
