use super::*;
use pretty_assertions::assert_eq;

use wok::{self, cmd, config::Config};

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b.toml")), expected_config("a-b-other.toml"))]
fn existing_branch(repo_sample: TestRepo, expected_config: String) {
    let mut actual_config = Config::load(&repo_sample.config_path()).unwrap();

    _run("git switch other", &repo_sample.repo_path).unwrap();

    cmd::head::switch(&mut actual_config, &repo_sample.repo()).unwrap();

    assert_eq!(actual_config.dump().unwrap(), expected_config);
}
