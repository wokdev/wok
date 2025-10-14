use std::{io::Cursor, path::Path};

use rstest::*;

use wok_dev::{cmd, config};

use super::*;

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_all_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with --all
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false, // create
        true,  // all
        None,  // branch
        &[],   // repos
    )
    .unwrap();

    assert!(config_changed);
    let repo_entry = actual_config
        .repos
        .iter()
        .find(|r| r.path == Path::new("sub-a"))
        .unwrap();
    assert_eq!(repo_entry.head, "main");

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories to branch 'main'"));
    assert!(
        output_str.contains("- 'sub-a':")
            && (output_str.contains("switched to 'main'")
                || output_str.contains("already on 'main'"))
    );
    assert!(output_str.contains("Locking submodule state"));
    assert!(output_str.contains("Successfully switched and locked 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn switch_all_skips_configured_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,
        true,
        None,
        &[],
    )
    .unwrap();

    assert!(config_changed);
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories to branch 'main'"));
    assert!(!output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b':"));
    assert!(output_str.contains("Successfully switched and locked 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn switch_all_includes_explicit_repo_overrides_skip(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,
        true,
        None,
        &[std::path::PathBuf::from("sub-a")],
    )
    .unwrap();

    assert!(config_changed);
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 2 repositories to branch 'main'"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b':"));
    assert!(output_str.contains("Successfully switched and locked 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_specific_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with specific repo
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,                                // create
        false,                                // all
        None,                                 // branch
        &[std::path::PathBuf::from("sub-a")], // repos
    )
    .unwrap();

    assert!(config_changed);
    let repo_entry = actual_config
        .repos
        .iter()
        .find(|r| r.path == Path::new("sub-a"))
        .unwrap();
    assert_eq!(repo_entry.head, "main");

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories to branch 'main'"));
    assert!(
        output_str.contains("- 'sub-a':")
            && (output_str.contains("switched to 'main'")
                || output_str.contains("already on 'main'"))
    );
    assert!(output_str.contains("Successfully switched and locked 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_with_create_option(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with --create and a new branch name
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        true,                   // create
        false,                  // all
        Some("feature-branch"), // branch
        &[],                    // repos
    )
    .unwrap();

    assert!(config_changed);
    let repo_entry = actual_config
        .repos
        .iter()
        .find(|r| r.path == Path::new("sub-a"))
        .unwrap();
    assert_eq!(repo_entry.head, "feature-branch");

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories to branch 'feature-branch'"));
    assert!(output_str.contains("- 'sub-a': created and switched to 'feature-branch'"));
    assert!(output_str.contains("Successfully switched and locked 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_with_branch_option(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    _run(
        "git switch -c develop",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();
    _run("git switch main", &repo_sample.subrepo_paths["sub-a"]).unwrap();

    // Run the switch command with --branch option
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,           // create
        false,           // all
        Some("develop"), // branch
        &[],             // repos
    )
    .unwrap();

    assert!(config_changed);
    let repo_entry = actual_config
        .repos
        .iter()
        .find(|r| r.path == Path::new("sub-a"))
        .unwrap();
    assert_eq!(repo_entry.head, "develop");

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories to branch 'develop'"));
    assert!(output_str.contains("Successfully switched and locked 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_all_repos_moves_repo_when_cached_head_matches_target(
    repo_sample: TestRepo,
) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    _run("git switch -c test", &repo_sample.repo_path).unwrap();
    _run(
        "git switch -c test",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();
    _run("git switch main", &repo_sample.subrepo_paths["sub-a"]).unwrap();

    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false, // create
        true,  // all
        None,  // branch
        &[],   // repos
    )
    .unwrap();

    assert!(config_changed);
    let repo_entry = actual_config
        .repos
        .iter()
        .find(|r| r.path == Path::new("sub-a"))
        .unwrap();
    assert_eq!(repo_entry.head, "test");

    let subrepo_branch = _run(
        "git branch --show-current",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();

    assert_eq!(subrepo_branch.trim(), "test");
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn switch_with_no_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with no repos configured
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false, // create
        false, // all
        None,  // branch
        &[],   // repos
    )
    .unwrap();

    assert!(!config_changed);
    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("No repositories to switch"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_nonexistent_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with a non-existent repo
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,                                      // create
        false,                                      // all
        None,                                       // branch
        &[std::path::PathBuf::from("nonexistent")], // repos
    )
    .unwrap();

    assert!(!config_changed);
    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("No repositories to switch"));
}
