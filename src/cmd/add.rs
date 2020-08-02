use git2;
use std::path;

pub fn add(
    state: &mut crate::State,
    git_url: String,
    module_path: path::PathBuf,
) -> Result<(), crate::Error> {
    eprintln!("{:?}", state);
    eprintln!("{:?}", git_url);
    eprintln!("{:?}", module_path);

    Ok(())
}
