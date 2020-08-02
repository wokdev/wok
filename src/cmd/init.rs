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

    #[fixture]
    fn data_dir() -> PathBuf {
        (PathBuf::from(env!("CARGO_MANIFEST_DIR"))).join("tests/data")
    }

    struct TestRepo {
        _temp_dir: assert_fs::TempDir,
        repo_path: PathBuf,
        cwd: PathBuf,
    }
    impl TestRepo {
        fn new(data_dir: &PathBuf, repo_name: &str) -> Self {
            let temp_dir = assert_fs::TempDir::new().unwrap();
            temp_dir
                .copy_from(
                    data_dir.join("repos"),
                    &[&(String::from(repo_name) + "/**")],
                )
                .unwrap();
            let repo_path = temp_dir.path().join(repo_name);
            let git_files = Self::find_git_files(&repo_path);
            for git_file in git_files {
                fs::rename(&git_file, &git_file.parent().unwrap().join(".git"))
                    .unwrap();
            }
            let cwd = env::current_dir().unwrap();
            env::set_current_dir(&repo_path).unwrap();
            Self {
                _temp_dir: temp_dir,
                repo_path,
                cwd,
            }
        }

        fn find_git_files(path: &PathBuf) -> Vec<PathBuf> {
            let mut git_files = vec![];
            for entry in fs::read_dir(&path).unwrap() {
                let entry_path: PathBuf = entry.unwrap().path();
                if entry_path.file_name().unwrap() == "_git" {
                    git_files.push(entry_path);
                } else if entry_path.is_dir() {
                    git_files.extend(Self::find_git_files(&entry_path))
                };
            }
            git_files
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            env::set_current_dir(&self.cwd).unwrap();
        }
    }

    #[fixture(repo_name = "")]
    fn repo_sample(repo_name: &str, data_dir: PathBuf) -> TestRepo {
        TestRepo::new(&data_dir, repo_name)
    }

    #[fixture(config_name = "")]
    fn expected_config(config_name: &str, data_dir: PathBuf) -> String {
        fs::read_to_string(data_dir.join("configs").join(config_name)).unwrap()
    }

    // TODO: see https://github.com/la10736/rstest/issues/29
    #[rstest(repo_sample("simple"), expected_config("simple.yml"))]
    #[serial]
    fn in_a_single_repo_using_defaults(repo_sample: TestRepo, expected_config: String) {
        let actual_config = init(None, false).unwrap().dump().unwrap();
        assert_eq!(actual_config, expected_config);
    }

    #[rstest(repo_sample("no-root"), expected_config("simple.yml"))]
    #[serial]
    fn in_a_rootless_repo_using_defaults(
        repo_sample: TestRepo,
        expected_config: String,
    ) {
        let actual_config = init(None, false).unwrap().dump().unwrap();
        assert_eq!(actual_config, expected_config);
    }

    #[rstest(repo_sample("submodules"), expected_config("submodules.yml"))]
    #[serial]
    fn with_submodules_using_defaults(repo_sample: TestRepo, expected_config: String) {
        let actual_config = init(None, false).unwrap().dump().unwrap();
        assert_eq!(actual_config, expected_config);
    }

    #[rstest(repo_sample("submodules"), expected_config("simple.yml"))]
    #[serial]
    fn with_submodules_using_no_introspect(
        repo_sample: TestRepo,
        expected_config: String,
    ) {
        let actual_config = init(None, true).unwrap().dump().unwrap();
        assert_eq!(actual_config, expected_config);
    }

    #[rstest(repo_sample("simple"), expected_config("develop.yml"))]
    #[serial]
    fn simple_using_custom_main_branch(repo_sample: TestRepo, expected_config: String) {
        let test_repo = git2::Repository::open(&repo_sample.repo_path).unwrap();
        test_repo
            .branch(
                "develop",
                &test_repo.head().unwrap().peel_to_commit().unwrap(),
                false,
            )
            .unwrap();
        let actual_config = init(Some(String::from("develop")), false)
            .unwrap()
            .dump()
            .unwrap();
        assert_eq!(actual_config, expected_config);
    }

    #[rstest(repo_sample("submodules"), expected_config("develop_submodules.yml"))]
    #[serial]
    fn with_submodules_using_custom_main_branch(
        repo_sample: TestRepo,
        expected_config: String,
    ) {
        let test_repo = git2::Repository::open(&repo_sample.repo_path).unwrap();
        test_repo
            .branch(
                "develop",
                &test_repo.head().unwrap().peel_to_commit().unwrap(),
                false,
            )
            .unwrap();
        let actual_config = init(Some(String::from("develop")), false)
            .unwrap()
            .dump()
            .unwrap();
        assert_eq!(actual_config, expected_config);
    }
}
