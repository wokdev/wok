use std::path;

pub fn add(
    state: &mut crate::State,
    git_url: String,
    mut module_path: path::PathBuf,
) -> Result<(), crate::Error> {
    let umbrella_workdir = state.umbrella.workdir().unwrap();

    if !module_path.is_absolute() {
        module_path = umbrella_workdir.join(module_path)
    }
    if !module_path.starts_with(umbrella_workdir) {
        return Err(crate::Error::new(String::from(
            "Submodule path is outside the repo workdir!",
        )));
    }
    let relative_module_path = module_path.strip_prefix(umbrella_workdir).unwrap();

    let mut added_submodule = state
        .umbrella
        .submodule(&git_url, relative_module_path, true)
        .map_err(|e| crate::Error::from(&e))?;
    added_submodule.open().map_err(|e| crate::Error::from(&e))?;
    let added_submodule_repo = added_submodule
        .clone(Some(&mut git2::SubmoduleUpdateOptions::default()))
        .map_err(|e| crate::Error::from(&e))?;
    added_submodule
        .add_finalize()
        .map_err(|e| crate::Error::from(&e))?;

    state.config.repos.push(crate::config::Repo {
        url: Some(String::from(git_url)),
        path: path::PathBuf::from(added_submodule.path()),
        ref_: String::from(added_submodule_repo.head().unwrap().shorthand().unwrap()),
    });

    Ok(())
}
