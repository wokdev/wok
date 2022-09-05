use super::*;
use pretty_assertions::assert_eq;

use wok::{self, cmd, config::Config};

#[rstest(repo_sample(vec![], Some("rm/simple-in.toml")), expected_config("rm/simple-out.toml"))]
fn in_a_single_repo(repo_sample: TestRepo, expected_config: String) {
    let mut actual_config = Config::load(&repo_sample.config_path()).unwrap();
    let submodule_path = repo_sample.add_submodule("sub-a");

    cmd::rm(&mut actual_config, &submodule_path).unwrap();

    assert_eq!(actual_config.dump().unwrap(), expected_config);
}
