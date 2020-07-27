use git2;
use std::path::PathBuf;

pub fn init(
    main_branch: Option<String>,
    no_introspect: bool,
) -> Result<crate::Config, crate::Error> {
    let umbrella_repo =
        git2::Repository::open_from_env().map_err(|e| crate::Error::from(&e))?;
    eprintln!(
        "Found git repo at `{}`",
        umbrella_repo.workdir().unwrap().to_string_lossy()
    );

    let main_branch = match main_branch {
        Some(branch) => branch,
        None => umbrella_repo
            .head()
            .map_err(|e| crate::Error::from(&e))?
            .shorthand()
            .unwrap()
            .to_string(),
    };
    eprintln!("Using main branch `{}`", main_branch);

    let submodules = match no_introspect {
        true => {
            eprintln!("Skipping submodule introspection.");
            vec![]
        }
        false => umbrella_repo
            .submodules()
            .map_err(|e| crate::Error::from(&e))?,
    };

    let config = crate::Config {
        version: String::from(crate::CONFIG_CURRENT_VERSION),
        ref_: main_branch.clone(),
        repos: submodules
            .into_iter()
            .map(|submodule| {
                let path = PathBuf::from(submodule.path());
                eprintln!("Found submodule at `{}`", path.to_string_lossy());
                crate::config::Repo {
                    url: submodule.url().map(|url| String::from(url)),
                    path,
                    ref_: main_branch.clone(),
                }
            })
            .collect(),
    };

    Ok(config)
}
