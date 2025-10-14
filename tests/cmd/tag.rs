use std::io::Cursor;

use rstest::*;

use wok_dev::{cmd, config};

use super::*;

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_list_all_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command without --create (list mode) with --all
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        None,  // tag_name
        false, // sign
        false, // push
        true,  // all
        &[],   // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Listing tags in 1 repositories"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_list_specific_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command without --create (list mode) with specific repo
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        None,                                 // tag_name
        false,                                // sign
        false,                                // push
        false,                                // all
        &[std::path::PathBuf::from("sub-a")], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Listing tags in 1 repositories"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_create_all_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command with --create
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        Some("v1.0.0"), // tag_name
        false,          // sign
        false,          // push
        true,           // all
        &[],            // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 1 repositories"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_create_specific_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command with --create and specific repo
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        Some("v1.0.0"),                       // tag_name
        false,                                // sign
        false,                                // push
        false,                                // all
        &[std::path::PathBuf::from("sub-a")], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 1 repositories"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_create_with_sign(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command with --create and --sign
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        Some("v1.0.0"), // tag_name
        true,           // sign
        false,          // push
        true,           // all
        &[],            // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 1 repositories"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_create_with_push(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command with --create and --push
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        Some("v1.0.0"), // tag_name
        false,          // sign
        true,           // push
        true,           // all
        &[],            // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 1 repositories"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("Pushing tags to remotes"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn tag_with_no_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command with no repos configured
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        None,  // tag_name
        false, // sign
        false, // push
        false, // all
        &[],   // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("No repositories to tag"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_nonexistent_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command with a non-existent repo
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        None,                                       // tag_name
        false,                                      // sign
        false,                                      // push
        false,                                      // all
        &[std::path::PathBuf::from("nonexistent")], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("No repositories to tag"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b.toml")))]
fn tag_multiple_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command with multiple repos
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        Some("v1.0.0"), // tag_name
        false,          // sign
        false,          // push
        true,           // all
        &[],            // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 2 repositories"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-b': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b.toml")))]
fn tag_specific_multiple_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the tag command with specific multiple repos
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        Some("v1.0.0"), // tag_name
        false,          // sign
        false,          // push
        false,          // all
        &[
            std::path::PathBuf::from("sub-a"),
            std::path::PathBuf::from("sub-b"),
        ], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 2 repositories"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-b': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn tag_all_skips_configured_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        Some("v1.0.0"),
        false,
        false,
        true,
        &[],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 1 repositories"));
    assert!(!output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn tag_all_includes_explicit_repo_overrides_skip(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        Some("v1.0.0"),
        false,
        false,
        true,
        &[std::path::PathBuf::from("sub-a")],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 2 repositories"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-b': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
}
