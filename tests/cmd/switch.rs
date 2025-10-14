use std::io::Cursor;

use rstest::*;

use wok_dev::{cmd, config};

use super::*;

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_all_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with --all
    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false, // create
        true,  // all
        None,  // branch
        &[],   // repos
    )
    .unwrap();

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

    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,
        true,
        None,
        &[],
    )
    .unwrap();

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

    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,
        true,
        None,
        &[std::path::PathBuf::from("sub-a")],
    )
    .unwrap();

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
    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,                                // create
        false,                                // all
        None,                                 // branch
        &[std::path::PathBuf::from("sub-a")], // repos
    )
    .unwrap();

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
    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        true,                   // create
        false,                  // all
        Some("feature-branch"), // branch
        &[],                    // repos
    )
    .unwrap();

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

    // Run the switch command with --branch option
    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,           // create
        false,           // all
        Some("develop"), // branch
        &[],             // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories to branch 'develop'"));
    assert!(output_str.contains("Successfully switched and locked 1 repositories"));
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn switch_with_no_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with no repos configured
    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false, // create
        false, // all
        None,  // branch
        &[],   // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("No repositories to switch"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_nonexistent_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with a non-existent repo
    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,                                      // create
        false,                                      // all
        None,                                       // branch
        &[std::path::PathBuf::from("nonexistent")], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("No repositories to switch"));
}
