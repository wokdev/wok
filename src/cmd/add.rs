use git2;
use std::path;

pub fn add(
    config: crate::Config,
    git_url: String,
    module_path: path::PathBuf,
) -> Result<crate::Config, crate::Error> {
    eprintln!("{:?}", config);
    eprintln!("{:?}", git_url);
    eprintln!("{:?}", module_path);

    // git2::Repository::clone_recurse(&git_url, module_path).unwrap();
    Ok(config)
}
