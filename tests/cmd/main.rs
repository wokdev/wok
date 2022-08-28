use anyhow::*;
use rstest::*;
use std::{collections::HashMap, env, fs, path::PathBuf, process};

// mod add;
mod init;

#[fixture]
fn data_dir() -> PathBuf {
    (PathBuf::from(env!("CARGO_MANIFEST_DIR"))).join("tests/data")
}

#[fixture(repo_name = "", sub_repo_names = vec![])]
fn repo_sample(repo_name: &str, sub_repo_names: Vec<&str>) -> TestRepo {
    TestRepo::new(repo_name, sub_repo_names)
}

#[fixture(config_name = "")]
fn expected_config(config_name: &str, data_dir: PathBuf) -> String {
    fs::read_to_string(data_dir.join(config_name)).unwrap()
}

struct TestRepo {
    _temp_dir: assert_fs::TempDir,
    repo_path: PathBuf,
    subrepo_paths: HashMap<String, PathBuf>,
}
impl TestRepo {
    fn new(repo_name: &str, sub_repo_names: Vec<&str>) -> Self {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let repo_path = temp_dir.path().join(repo_name);

        fs::create_dir(&repo_path).unwrap();

        _run("git init", &repo_path).unwrap();
        _run(
            "git commit --allow-empty --allow-empty-message -m ''",
            &repo_path,
        )
        .unwrap();

        let mut subrepo_paths: HashMap<String, PathBuf> = HashMap::new();

        for sub_repo_name in sub_repo_names {
            let sub_repo_path = repo_path.join(&sub_repo_name);
            _run(&format!("mkdir {}", sub_repo_name), &repo_path).unwrap();
            _run("git init", &sub_repo_path).unwrap();
            _run(
                "git commit --allow-empty --allow-empty-message -m ''",
                &sub_repo_path,
            )
            .unwrap();
            _run(
                &format!("git submodule add ./{} {}", &sub_repo_name, &sub_repo_name),
                &repo_path,
            )
            .unwrap();
            subrepo_paths.insert(String::from(sub_repo_name), sub_repo_path);
        }

        Self {
            _temp_dir: temp_dir,
            repo_path,
            subrepo_paths,
        }
    }
}

fn _run(cmd: &str, cwd: &PathBuf) -> Result<String> {
    let argv = shell_words::split(cmd).unwrap();
    let output = process::Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(cwd)
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
