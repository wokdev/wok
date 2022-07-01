use rstest::*;
use std::{env, fs, path::PathBuf, process};

mod add;
mod init;

#[fixture]
fn data_dir() -> PathBuf {
    (PathBuf::from(env!("CARGO_MANIFEST_DIR"))).join("tests/data")
}

#[fixture(repo_name = "", sub_repo_names = vec![], rootless = false)]
fn repo_sample(
    repo_name: &str,
    sub_repo_names: Vec<&str>,
    rootless: bool,
    data_dir: PathBuf,
) -> TestRepo {
    TestRepo::new(&data_dir, repo_name, sub_repo_names, rootless)
}

#[fixture(config_name = "")]
fn expected_config(config_name: &str, data_dir: PathBuf) -> String {
    fs::read_to_string(data_dir.join(config_name)).unwrap()
}

struct TestRepo {
    _temp_dir: assert_fs::TempDir,
    repo_path: PathBuf,
}
impl TestRepo {
    fn new(
        data_dir: &PathBuf,
        repo_name: &str,
        sub_repo_names: Vec<&str>,
        rootless: bool,
    ) -> Self {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let repo_path = temp_dir.path().join(repo_name);

        fs::create_dir(&repo_path).unwrap();

        _run("git init", &repo_path);
        if !rootless {
            _run(
                "git commit --allow-empty --allow-empty-message -m ''",
                &repo_path,
            );
        }

        for sub_repo_name in sub_repo_names {
            _run(
                &format!("git submodule add ../{} {}", &repo_name, &sub_repo_name),
                &repo_path,
            );
        }

        Self {
            _temp_dir: temp_dir,
            repo_path,
        }
    }
}

fn _run(cmd: &str, cwd: &PathBuf) {
    let argv = shell_words::split(cmd).unwrap();
    process::Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(cwd)
        // .stdout(process::Stdio::inherit())
        // .stderr(process::Stdio::inherit())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
