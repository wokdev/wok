use super::*;
use pretty_assertions::assert_eq;
use std::path;
use wok::{self, cmd};

#[rstest(repo_sample("simple"), expected_config("add/simple.yaml"))]
fn local_repo(repo_sample: TestRepo, expected_config: String) {
    let config = cmd::init(&repo_sample.repo_path, None, false).unwrap();
    config
        .save(&repo_sample.repo_path.join("wok.yaml"))
        .unwrap();
    let mut state = wok::State::new(&repo_sample.repo_path.join("wok.yaml")).unwrap();
    cmd::add(
        &mut state,
        format!(
            "../{}",
            repo_sample.repo_path.file_name().unwrap().to_string_lossy()
        ),
        path::PathBuf::from("added"),
    )
    .unwrap();
    let actual_config = state.into_config().dump().unwrap();
    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample("submodules", vec!["sub-a", "sub-b"]), expected_config("add/submodules.yaml"))]
fn local_repo_to_existing_submodules(repo_sample: TestRepo, expected_config: String) {
    let config = cmd::init(&repo_sample.repo_path, None, false).unwrap();
    config
        .save(&repo_sample.repo_path.join("wok.yaml"))
        .unwrap();
    let mut state = wok::State::new(&repo_sample.repo_path.join("wok.yaml")).unwrap();
    cmd::add(
        &mut state,
        format!(
            "../{}",
            repo_sample.repo_path.file_name().unwrap().to_string_lossy()
        ),
        path::PathBuf::from("added"),
    )
    .unwrap();
    let actual_config = state.into_config().dump().unwrap();
    assert_eq!(actual_config, expected_config);
}
