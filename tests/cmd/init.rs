use pretty_assertions::assert_eq;
use rstest::*;

use wok_dev::{cmd, config::Config};

use super::*;

// TODO: see https://github.com/la10736/rstest/issues/29
#[rstest(repo_sample(), expected_config("empty.toml"))]
fn in_a_single_repo(repo_sample: TestRepo, expected_config: String) {
    let config_path = repo_sample.config_path();

    cmd::init(&config_path, &repo_sample.repo(), false).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"]), expected_config("a-b.toml"))]
fn with_submodules(repo_sample: TestRepo, expected_config: String) {
    let config_path = repo_sample.config_path();

    cmd::init(&config_path, &repo_sample.repo(), false).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample(vec!["deps/sub-a"]), expected_config("deps-a.toml"))]
fn with_complex_submodule_path(repo_sample: TestRepo, expected_config: String) {
    let config_path = repo_sample.config_path();

    cmd::init(&config_path, &repo_sample.repo(), false).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"]), expected_config("a-b.toml"))]
fn with_sync(repo_sample: TestRepo, expected_config: String) {
    _run(
        "git switch -c other-branch",
        repo_sample.subrepo_paths.get("sub-a").unwrap(),
    )
    .unwrap();
    let config_path = repo_sample.config_path();

    cmd::init(&config_path, &repo_sample.repo(), true).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);

    let sub_a_branch = _run(
        "git branch --show-current",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();

    assert_eq!(sub_a_branch.trim(), String::from("main"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"]), expected_config("a-b.toml"))]
fn without_sync(repo_sample: TestRepo, expected_config: String) {
    _run(
        "git switch -c other-branch",
        repo_sample.subrepo_paths.get("sub-a").unwrap(),
    )
    .unwrap();
    let config_path = repo_sample.config_path();

    cmd::init(&config_path, &repo_sample.repo(), false).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);

    let sub_a_branch = _run(
        "git branch --show-current",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();

    assert_eq!(sub_a_branch.trim(), String::from("other-branch"));
}
