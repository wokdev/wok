use pretty_assertions::assert_eq;
use rstest::*;

use super::*;
use wok::{self, cmd};

#[rstest(repo_sample("simple"), expected_config("add/simple.yml"))]
fn local_repo(repo_sample: TestRepo, expected_config: String) {
    let config = cmd::init(&repo_sample.repo_path, None, false).unwrap();
    config.save(&repo_sample.repo_path.join("wok.yml")).unwrap();
    let mut state = wok::State::new(&repo_sample.repo_path.join("wok.yml")).unwrap();
    cmd::add(
        &mut state,
        &repo_sample.repo_path.to_str().unwrap(),
        &repo_sample.repo_path.join("added"),
    )
    .unwrap();
    let actual_config = state.into_config().dump().unwrap();
    assert_eq!(actual_config, expected_config);
}

#[rstest(repo_sample("submodules"), expected_config("add/submodules.yml"))]
fn local_repo_to_existing_submodules(repo_sample: TestRepo, expected_config: String) {
    let config = cmd::init(&repo_sample.repo_path, None, false).unwrap();
    config.save(&repo_sample.repo_path.join("wok.yml")).unwrap();
    let mut state = wok::State::new(&repo_sample.repo_path.join("wok.yml")).unwrap();
    cmd::add(
        &mut state,
        &repo_sample.repo_path.to_str().unwrap(),
        &repo_sample.repo_path.join("added"),
    )
    .unwrap();
    let actual_config = state.into_config().dump().unwrap();
    assert_eq!(actual_config, expected_config);
}
