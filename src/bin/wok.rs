use std::path::PathBuf;
use structopt::StructOpt;
use wok;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "wok",
    about = "Wok -- control several git repositories as a single project."
)]
struct Opt {
    /// Config path
    #[structopt(long, parse(from_os_str))]
    config_path: PathBuf,
}

fn main() -> Result<(), wok::Error> {
    let opt = Opt::from_args();
    assert!(opt.config_path.exists());

    let config = wok::Config::load(&opt.config_path).map_err(|e| wok::Error::from(&e))?;

    println!("{:?}", config);
    println!("{:}", config.save());

    Ok(())
}
