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

#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::prelude::*;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use std::env;
    use std::fs;

    #[fixture(config_file=PathBuf::from("tests/data/simple_config.yml"))]
    fn expected_config(config_file: PathBuf) -> String {
        fs::read_to_string(config_file).unwrap()
    }

    #[rstest]
    fn init_with_defaults_in_a_single_repo(expected_config: String) {
        let repo_dir = assert_fs::TempDir::new().unwrap();
        repo_dir
            .copy_from("tests/data/repos", &["simple/**"])
            .unwrap();
        let repo_path = repo_dir.path().join("simple");
        fs::rename(repo_path.join("_git"), repo_path.join(".git")).unwrap();
        let cwd = env::current_dir().unwrap();
        env::set_current_dir(&repo_path).unwrap();

        let actual_config = init(None, false).unwrap().dump().unwrap();
        assert_eq!(actual_config, expected_config);

        env::set_current_dir(cwd).unwrap();
        repo_dir.close().unwrap();
    }
}
