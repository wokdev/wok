use crate::{Config, Error};
use git2;

pub fn init(main_branch: Option<String>, no_introspect: bool) -> Result<(), Error> {
    eprintln!("{:?}, {:?}", main_branch, no_introspect);

    let umbrella_repo =
        git2::Repository::open_from_env().map_err(|e| Error::from(&e))?;
    let main_branch = match main_branch {
        Some(branch) => branch,
        None => umbrella_repo
            .head()
            .map_err(|e| Error::from(&e))?
            .shorthand()
            .unwrap()
            .to_string(),
    };

    eprintln!("{:?}", main_branch);

    Ok(())
}
