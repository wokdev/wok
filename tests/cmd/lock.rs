use std::io::Cursor;

use pretty_assertions::assert_eq;
use rstest::*;

use git_wok::{cmd, config};

use super::*;

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn lock_with_no_changes_skips_commit(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Commit the initial submodule setup
    _run("git add .", repo_sample.repo_path()).unwrap();
    _run("git commit -m 'Initial setup'", repo_sample.repo_path()).unwrap();

    // Get initial commit count
    let repo = repo_sample.repo();
    let mut revwalk = repo.git_repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let initial_count = revwalk.count();

    // Run the lock command without making any changes
    cmd::lock(&mut actual_config, &repo_sample.repo(), &mut output).unwrap();

    // Check the output indicates no changes
    assert_eq!(
        String::from_utf8_lossy(output.get_ref()),
        "No submodule changes detected; nothing to lock\n"
    );

    // Verify that no new commit was created
    let mut revwalk = repo.git_repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let new_count = revwalk.count();
    assert_eq!(new_count, initial_count);
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn lock_with_no_submodules(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Get initial commit count
    let repo = repo_sample.repo();
    let mut revwalk = repo.git_repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let initial_count = revwalk.count();

    // Run the lock command with no submodules
    cmd::lock(&mut actual_config, &repo_sample.repo(), &mut output).unwrap();

    // Check the output indicates no changes
    assert_eq!(
        String::from_utf8_lossy(output.get_ref()),
        "No submodule changes detected; nothing to lock\n"
    );

    // Verify that no new commit was created
    let mut revwalk = repo.git_repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let new_count = revwalk.count();
    assert_eq!(new_count, initial_count);
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn lock_with_changed_submodule(repo_sample: TestRepo) {
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Commit the initial submodule setup
    _run("git add .", repo_sample.repo_path()).unwrap();
    _run("git commit -m 'Initial setup'", repo_sample.repo_path()).unwrap();

    // Get initial commit count
    let repo = repo_sample.repo();
    let mut revwalk = repo.git_repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let initial_count = revwalk.count();

    // Make a change in submodule
    let subrepo_path = repo_sample.subrepo_path("sub-a").unwrap();
    _run("git commit --allow-empty -m 'Test commit message for submodule that is quite long and should be truncated properly'", subrepo_path).unwrap();

    let mut output = Cursor::new(Vec::new());

    // Run the lock command
    cmd::lock(&mut actual_config, &repo, &mut output).unwrap();

    // Check the output
    assert_eq!(
        String::from_utf8_lossy(output.get_ref()),
        "Locked submodule state\n"
    );

    // Verify that a new commit was created
    let mut revwalk = repo.git_repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let new_count = revwalk.count();
    assert_eq!(new_count, initial_count + 1);

    // Verify commit message includes submodule info
    let head = repo.git_repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let message = commit.message().unwrap();

    assert!(message.contains("Lock submodule state"));
    assert!(message.contains("Changed submodules:"));
    assert!(message.contains("sub-a:"));
    // Check for truncated message (truncated at 47 chars + "...")
    assert!(
        message.contains("sub-a: Test commit message for submodule that is quite...")
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn lock_with_short_commit_message(repo_sample: TestRepo) {
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Commit the initial submodule setup
    _run("git add .", repo_sample.repo_path()).unwrap();
    _run("git commit -m 'Initial setup'", repo_sample.repo_path()).unwrap();

    // Make a change in submodule with short message
    let subrepo_path = repo_sample.subrepo_path("sub-a").unwrap();
    _run("git commit --allow-empty -m 'Short message'", subrepo_path).unwrap();

    let mut output = Cursor::new(Vec::new());

    // Run the lock command
    let repo = repo_sample.repo();
    cmd::lock(&mut actual_config, &repo, &mut output).unwrap();

    // Verify commit message includes full submodule message (not truncated)
    let head = repo.git_repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let message = commit.message().unwrap();

    assert!(message.contains("Lock submodule state"));
    assert!(message.contains("Changed submodules:"));
    assert!(message.contains("sub-a: Short message"));
    // Should not have ellipsis
    assert!(!message.contains("..."));
}
