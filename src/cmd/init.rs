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
        Some(main_branch) => umbrella_repo
            .find_branch(&main_branch, git2::BranchType::Local)
            .map_err(|e| crate::Error::from(&e))?
            .name()
            .map_err(|e| crate::Error::from(&e))?
            .unwrap()
            .to_string(),
        None => {
            if umbrella_repo
                .head_detached()
                .expect("Error finding umbrella repo head")
            {
                return Err(crate::Error::new(
                    "Umbrella repo head is detached, provide main branch name \
                     explicitly!"
                        .to_string(),
                ));
            };
            let head_ref = umbrella_repo
                .find_reference("HEAD")
                .map_err(|e| crate::Error::from(&e))?;
            let head_target = head_ref.symbolic_target().unwrap();
            if !head_target.starts_with("refs/heads/") {
                return Err(crate::Error::new(
                    "HEAD of umbrella repo doesn't point to a local branch".to_string(),
                ));
            }
            head_target[11..].to_string()
        },
    };
    eprintln!("Using main branch `{}`", &main_branch);

    let submodules = match no_introspect {
        true => {
            eprintln!("Skipping submodule introspection.");
            vec![]
        },
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
