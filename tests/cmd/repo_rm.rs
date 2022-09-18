use super::*;
use pretty_assertions::assert_eq;

use wok::{self, cmd, config::Config};

#[rstest(repo_sample(vec![], Some("a.toml")), expected_config("empty.toml"))]
fn in_a_single_repo(repo_sample: TestRepo, expected_config: String) {
    let mut actual_config = Config::load(&repo_sample.config_path()).unwrap();
    let submodule_path = repo_sample.add_submodule("sub-a");

    cmd::repo::rm(&mut actual_config, &submodule_path).unwrap();

    assert_eq!(actual_config.dump().unwrap(), expected_config);
}
