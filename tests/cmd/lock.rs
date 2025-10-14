use std::io::Cursor;

use pretty_assertions::assert_eq;
use rstest::*;

use git_wok::{cmd, config};

use super::*;

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn lock_submodule_state(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the lock command
    cmd::lock(&mut actual_config, &repo_sample.repo(), &mut output).unwrap();

    // Check the output
    assert_eq!(
        String::from_utf8_lossy(output.get_ref()),
        "Locked submodule state\n"
    );

    // Verify that a commit was created
    let repo = repo_sample.repo();
    let head = repo.git_repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let message = commit.message().unwrap();

    // The commit message should indicate it's a lock commit
    assert!(message.contains("Lock submodule state"));
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn lock_with_no_submodules(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the lock command with no submodules
    cmd::lock(&mut actual_config, &repo_sample.repo(), &mut output).unwrap();

    // Check the output
    assert_eq!(
        String::from_utf8_lossy(output.get_ref()),
        "Locked submodule state\n"
    );

    // Verify that a commit was created even with no submodules
    let repo = repo_sample.repo();
    let head = repo.git_repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let message = commit.message().unwrap();

    // The commit message should indicate it's a lock commit
    assert!(message.contains("Lock submodule state"));
}
