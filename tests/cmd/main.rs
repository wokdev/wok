use assert_fs::prelude::*;
use rstest::*;
use std::{env, fs, path::PathBuf};

mod init;

#[fixture]
fn data_dir() -> PathBuf {
    (PathBuf::from(env!("CARGO_MANIFEST_DIR"))).join("tests/data")
}

#[fixture(repo_name = "")]
fn repo_sample(repo_name: &str, data_dir: PathBuf) -> TestRepo {
    TestRepo::new(&data_dir, repo_name)
}

#[fixture(config_name = "")]
fn expected_config(config_name: &str, data_dir: PathBuf) -> String {
    fs::read_to_string(data_dir.join("configs").join(config_name)).unwrap()
}

struct TestRepo {
    _temp_dir: assert_fs::TempDir,
    repo_path: PathBuf,
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
            fs::rename(&git_file, &git_file.parent().unwrap().join(".git")).unwrap();
        }
        Self {
            _temp_dir: temp_dir,
            repo_path,
        }
    }

    fn find_git_files(search_path: &PathBuf) -> Vec<PathBuf> {
        let mut git_files = vec![];
        for entry in fs::read_dir(&search_path).unwrap() {
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
