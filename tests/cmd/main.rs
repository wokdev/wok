use anyhow::*;
use rstest::*;
use std::{
    collections::HashMap,
    env, fs,
    path::{self, PathBuf},
    process,
};
use wok_dev::{repo, DEFAULT_CONFIG_NAME};

mod head_switch;
mod init;
mod repo_add;
mod repo_rm;

#[fixture]
fn data_dir() -> PathBuf {
    (PathBuf::from(env!("CARGO_MANIFEST_DIR"))).join("tests/data")
}

#[fixture]
fn configs_dir(data_dir: PathBuf) -> PathBuf {
    data_dir.join("configs")
}

#[fixture(subrepo_names = vec![], with_config = None)]
fn repo_sample(
    subrepo_names: Vec<&str>,
    with_config: Option<&str>,
    configs_dir: PathBuf,
) -> TestRepo {
    TestRepo::new(
        subrepo_names,
        with_config.map(|config_name| configs_dir.join(config_name)),
    )
}

#[fixture(config_name = "")]
fn expected_config(config_name: &str, configs_dir: PathBuf) -> String {
    fs::read_to_string(configs_dir.join(config_name)).unwrap()
}

struct TestRepo {
    _temp_dir: assert_fs::TempDir,
    repo_path: PathBuf,
    subrepo_paths: HashMap<String, PathBuf>,
}
impl TestRepo {
    fn new(subrepo_names: Vec<&str>, config_name: Option<PathBuf>) -> Self {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_owned();

        Self::init_repo(&repo_path);

        let mut subrepo_paths: HashMap<String, PathBuf> = HashMap::new();

        for subrepo_name in subrepo_names {
            let subrepo_path = Self::create_submodule(&repo_path, subrepo_name);
            subrepo_paths.insert(String::from(subrepo_name), subrepo_path);
        }

        if let Some(config_path) = config_name {
            fs::copy(config_path, repo_path.join(DEFAULT_CONFIG_NAME)).unwrap();
        }

        Self {
            _temp_dir: temp_dir,
            repo_path,
            subrepo_paths,
        }
    }

    pub fn repo(&self) -> repo::Repo {
        repo::Repo::new(&self.repo_path, None).unwrap()
    }

    pub fn config_path(&self) -> path::PathBuf {
        self.repo_path.join(DEFAULT_CONFIG_NAME)
    }

    pub fn add_submodule(&self, subrepo_name: &str) -> path::PathBuf {
        Self::create_submodule(&self.repo_path, subrepo_name)
            .strip_prefix(&self.repo_path)
            .unwrap()
            .to_owned()
    }

    fn init_repo(repo_path: &PathBuf) {
        _run("git init -b main", repo_path).unwrap();
        _run("git config user.email 'test@localhost'", repo_path).unwrap();
        _run("git config user.name 'Test User'", repo_path).unwrap();
        _run(
            "git commit --allow-empty --allow-empty-message -m ''",
            repo_path,
        )
        .unwrap();
        _run("git branch -c other", repo_path).unwrap();
    }

    fn create_submodule(repo_path: &PathBuf, submodule_name: &str) -> PathBuf {
        let subrepo_path = repo_path.join(submodule_name);
        fs::create_dir_all(&subrepo_path).unwrap();
        Self::init_repo(&subrepo_path);
        _run(
            &format!(
                "git submodule add ./{submodule_name} {submodule_name}",
                submodule_name = submodule_name
            ),
            repo_path,
        )
        .unwrap();
        subrepo_path
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
