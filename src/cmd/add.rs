use git2;
use std::path;

pub fn add(
    state: &mut crate::State,
    git_url: &str,
    module_path: &path::Path,
) -> Result<(), crate::Error> {
    eprintln!("{:?}", state);
    eprintln!("{:?}", git_url);
    eprintln!("{:?}", module_path);

    Ok(())
}
