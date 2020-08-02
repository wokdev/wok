use pretty_assertions::assert_eq;
use rstest::*;
use serial_test::*;

use wok::cmd;

use super::*;

// TODO: see https://github.com/la10736/rstest/issues/29
#[rstest(repo_sample("simple"), expected_config("init/simple.yml"))]
#[serial]
fn in_a_single_repo_using_defaults(repo_sample: TestRepo, expected_config: String) {
    let actual_config = cmd::init(None, false).unwrap().dump().unwrap();
    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample("no-root"), expected_config("init/simple.yml"))]
#[serial]
fn in_a_rootless_repo_using_defaults(repo_sample: TestRepo, expected_config: String) {
    let actual_config = cmd::init(None, false).unwrap().dump().unwrap();
    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample("submodules"), expected_config("init/submodules.yml"))]
#[serial]
fn with_submodules_using_defaults(repo_sample: TestRepo, expected_config: String) {
    let actual_config = cmd::init(None, false).unwrap().dump().unwrap();
    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample("submodules"), expected_config("init/simple.yml"))]
#[serial]
fn with_submodules_using_no_introspect(repo_sample: TestRepo, expected_config: String) {
    let actual_config = cmd::init(None, true).unwrap().dump().unwrap();
    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample("simple"), expected_config("init/develop.yml"))]
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
    let actual_config = cmd::init(Some(String::from("develop")), false)
        .unwrap()
        .dump()
        .unwrap();
    assert_eq!(actual_config, expected_config);
}

#[rstest(
    repo_sample("submodules"),
    expected_config("init/develop_submodules.yml")
)]
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
    let actual_config = cmd::init(Some(String::from("develop")), false)
        .unwrap()
        .dump()
        .unwrap();
    assert_eq!(actual_config, expected_config);
}
