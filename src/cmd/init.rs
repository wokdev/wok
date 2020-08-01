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

#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::prelude::*;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use serial_test::*;
    use std::{env, fs};

    #[fixture(config_name = "")]
    fn expected_config(config_name: &str) -> String {
        fs::read_to_string(PathBuf::from("tests/data/configs").join(config_name))
            .unwrap()
    }

    #[rstest(expected_config("simple.yml"))]
    #[serial]
    fn in_a_single_repo_using_defaults(expected_config: String) {
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

    #[rstest(expected_config("simple.yml"))]
    #[serial]
    fn in_a_rootless_repo_using_defaults(expected_config: String) {
        let repo_dir = assert_fs::TempDir::new().unwrap();
        repo_dir
            .copy_from("tests/data/repos", &["no-root/**"])
            .unwrap();
        let repo_path = repo_dir.path().join("no-root");
        fs::rename(repo_path.join("_git"), repo_path.join(".git")).unwrap();
        let cwd = env::current_dir().unwrap();
        env::set_current_dir(&repo_path).unwrap();

        let actual_config = init(None, false).unwrap().dump().unwrap();
        assert_eq!(actual_config, expected_config);

        env::set_current_dir(cwd).unwrap();
        repo_dir.close().unwrap();
    }

    fn find_git_files(path: &PathBuf) -> Vec<PathBuf> {
        let mut git_files = vec![];
        for entry in fs::read_dir(&path).unwrap() {
            let entry_path: PathBuf = entry.unwrap().path();
            if entry_path.file_name().unwrap() == "_git" {
                git_files.push(entry_path);
            } else if entry_path.is_dir() {
                git_files.extend(find_git_files(&entry_path))
            };
        }
        git_files
    }

    #[rstest(expected_config("submodules.yml"))]
    #[serial]
    fn with_submodules_using_defaults(expected_config: String) {
        let repo_dir = assert_fs::TempDir::new().unwrap();
        repo_dir
            .copy_from("tests/data/repos", &["submodules/**"])
            .unwrap();
        let repo_path = repo_dir.path().join("submodules");
        let git_files = find_git_files(&repo_path);
        for git_file in git_files {
            fs::rename(&git_file, &git_file.parent().unwrap().join(".git")).unwrap();
        }
        let cwd = env::current_dir().unwrap();
        env::set_current_dir(&repo_path).unwrap();

        let actual_config = init(None, false).unwrap().dump().unwrap();
        assert_eq!(actual_config, expected_config);

        env::set_current_dir(cwd).unwrap();
        repo_dir.close().unwrap();
    }
}
