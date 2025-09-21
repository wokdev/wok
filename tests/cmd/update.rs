use std::io::Cursor;

use rstest::*;

use wok_dev::{cmd, config};

use super::*;

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn update_submodules(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the update command
    cmd::update(&mut actual_config, &repo_sample.repo(), &mut output).unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Updating submodules..."));
    assert!(output_str.contains("- 'sub-a': already up to date on 'main'"));
    assert!(output_str.contains("Updated submodule state committed"));

    // Verify that a commit was created
    let repo = repo_sample.repo();
    let head = repo.git_repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let message = commit.message().unwrap();

    // The commit message should indicate it's an update commit
    assert!(message.contains("Update submodules to latest"));
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn update_with_no_submodules(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the update command with no submodules
    cmd::update(&mut actual_config, &repo_sample.repo(), &mut output).unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Updating submodules..."));
    assert!(output_str.contains("Updated submodule state committed"));

    // Verify that a commit was created even with no submodules
    let repo = repo_sample.repo();
    let head = repo.git_repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let message = commit.message().unwrap();

    // The commit message should indicate it's an update commit
    assert!(message.contains("Update submodules to latest"));
}
