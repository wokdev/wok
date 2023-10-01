use std::io::Cursor;

use pretty_assertions::assert_eq;
use rstest::*;

use wok_dev::{cmd, config};

use super::*;

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn no_submodules_no_changes(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output).unwrap();

    assert_eq!(
        String::from_utf8_lossy(output.get_ref()),
        format!("On branch '{}', all clean\n", "main")
    );
}

fn _no_submodules_with_changes() {}
fn _with_submodules_branch_matches_no_changes() {}
fn _with_submodules_branch_matches_with_changes() {}
fn _with_submodules_branch_doesnt_match_no_changes() {}
fn _with_submodules_branch_doesnt_match_with_changes() {}
