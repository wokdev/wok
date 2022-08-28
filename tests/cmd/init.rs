use pretty_assertions::assert_eq;
use rstest::*;

use wok::{cmd, config::Config, DEFAULT_CONFIG_NAME};

use super::*;

// TODO: see https://github.com/la10736/rstest/issues/29
#[rstest(repo_sample("simple"), expected_config("init/simple.yaml"))]
fn in_a_single_repo(repo_sample: TestRepo, expected_config: String) {
    let config_path = repo_sample.repo_path.join(DEFAULT_CONFIG_NAME);

    cmd::init(&config_path, false).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample("submodules", vec!["sub-a", "sub-b"]), expected_config("init/submodules.yaml"))]
fn with_submodules(repo_sample: TestRepo, expected_config: String) {
    let config_path = repo_sample.repo_path.join(DEFAULT_CONFIG_NAME);

    cmd::init(&config_path, false).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample("submodules", vec!["sub-a", "sub-b"]), expected_config("init/submodules.yaml"))]
fn with_sync(repo_sample: TestRepo, expected_config: String) {
    _run(
        "git switch -c other-branch",
        &repo_sample.subrepo_paths.get("sub-a").unwrap(),
    )
    .unwrap();
    let config_path = repo_sample.repo_path.join(DEFAULT_CONFIG_NAME);

    cmd::init(&config_path, true).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);

    let sub_a_branch = _run(
        "git branch --show-current",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();

    assert_eq!(sub_a_branch.trim(), String::from("main"));
}

#[rstest(repo_sample("submodules", vec!["sub-a", "sub-b"]), expected_config("init/submodules.yaml"))]
fn without_sync(repo_sample: TestRepo, expected_config: String) {
    _run(
        "git switch -c other-branch",
        &repo_sample.subrepo_paths.get("sub-a").unwrap(),
    )
    .unwrap();
    let config_path = repo_sample.repo_path.join(DEFAULT_CONFIG_NAME);

    cmd::init(&config_path, false).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);

    let sub_a_branch = _run(
        "git branch --show-current",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();

    assert_eq!(sub_a_branch.trim(), String::from("other-branch"));
}
