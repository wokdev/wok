use super::*;
use pretty_assertions::assert_eq;

use wok::{self, cmd, config::Config};

#[rstest(repo_sample(vec![], Some("empty.toml")), expected_config("a.toml"))]
fn in_a_single_repo(repo_sample: TestRepo, expected_config: String) {
    let mut actual_config = Config::load(&repo_sample.config_path()).unwrap();
    let submodule_path = repo_sample.add_submodule("sub-a");

    cmd::repo::add(&mut actual_config, &repo_sample.repo(), &submodule_path).unwrap();

    assert_eq!(actual_config.dump().unwrap(), expected_config);
}
