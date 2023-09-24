use pretty_assertions::assert_eq;
use rstest::*;

use wok_dev::{cmd, config::Config};

use super::*;

#[rstest(repo_sample(), expected_config("empty.toml"))]
fn in_a_single_repo(repo_sample: TestRepo, expected_config: String) {
    let config_path = repo_sample.config_path();

    cmd::init(&config_path, &repo_sample.repo()).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"]), expected_config("a-b.toml"))]
fn with_submodules(repo_sample: TestRepo, expected_config: String) {
    let config_path = repo_sample.config_path();

    cmd::init(&config_path, &repo_sample.repo()).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample(vec!["deps/sub-a"]), expected_config("deps-a.toml"))]
fn with_complex_submodule_path(repo_sample: TestRepo, expected_config: String) {
    let config_path = repo_sample.config_path();

    cmd::init(&config_path, &repo_sample.repo()).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"]), expected_config("a-b.toml"))]
fn no_sync_on_init(repo_sample: TestRepo, expected_config: String) {
    _run(
        "git switch -c other-branch",
        repo_sample.subrepo_paths.get("sub-a").unwrap(),
    )
    .unwrap();
    let config_path = repo_sample.config_path();

    cmd::init(&config_path, &repo_sample.repo()).unwrap();
    let actual_config = Config::read(&config_path).unwrap();

    assert_eq!(actual_config, expected_config);

    let sub_a_branch = _run(
        "git branch --show-current",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();

    assert_eq!(sub_a_branch.trim(), String::from("other-branch"));
}

#[rstest(repo_sample(), expected_config("empty.toml"))]
fn fail_if_wokfile_already_exists(repo_sample: TestRepo, expected_config: String) {
    let config_path = repo_sample.config_path();

    fs::write(&config_path, expected_config).unwrap();

    let result_err = cmd::init(&config_path, &repo_sample.repo()).unwrap_err();
    let display_err = result_err.to_string();
    assert_eq!(
        display_err,
        format!("Wok file already exists at `{}`", config_path.display())
    );
}
