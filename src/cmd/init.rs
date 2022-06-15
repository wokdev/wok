use anyhow::{bail, Result};
use git2;
use std::path;

pub fn init(
    umbrella_path: &path::Path,
    main_branch: Option<String>,
    no_introspect: bool,
) -> Result<crate::Config> {
    let umbrella_repo = git2::Repository::open(umbrella_path)?;
    eprintln!(
        "Found git repo at `{}`",
        umbrella_repo.workdir().unwrap().to_string_lossy()
    );

    let main_branch = match main_branch {
        Some(main_branch) => umbrella_repo
            .find_branch(&main_branch, git2::BranchType::Local)?
            .name()?
            .unwrap()
            .to_string(),
        None => {
            if umbrella_repo
                .head_detached()
                .expect("Error finding umbrella repo head")
            {
                bail!("Umbrella repo head is detached, provide main branch name explicitly!");
            };
            let head_ref = umbrella_repo.find_reference("HEAD")?;
            let head_target = head_ref.symbolic_target().unwrap();
            if !head_target.starts_with("refs/heads/") {
                bail!("HEAD of umbrella repo doesn't point to a local branch");
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
        false => umbrella_repo.submodules()?,
    };

    let config = crate::Config {
        version: String::from(crate::CONFIG_CURRENT_VERSION),
        ref_: main_branch.clone(),
        repos: submodules
            .into_iter()
            .map(|submodule| {
                let module_path = path::PathBuf::from(submodule.path());
                eprintln!("Found submodule at `{}`", module_path.to_string_lossy());
                crate::config::Repo {
                    url: submodule.url().map(String::from),
                    path: module_path,
                    ref_: main_branch.clone(),
                }
            })
            .collect(),
    };

    Ok(config)
}
