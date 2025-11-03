use std::fs;
use std::io::Cursor;

use rstest::*;

use git_wok::{cmd, config};

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
        true,  // include umbrella
        &[],   // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Listing tags in 2 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_list_skips_umbrella_when_disabled(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        None,
        false,
        false,
        true,
        false,
        &[],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Listing tags in 1 repositories"));
    assert!(!output_str.contains("- 'umbrella':"));
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
        true,                                 // include umbrella
        &[std::path::PathBuf::from("sub-a")], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Listing tags in 2 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
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
        true,           // include umbrella
        &[],            // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 2 repositories"));
    assert!(output_str.contains("- 'umbrella': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
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
        true,                                 // include umbrella
        &[std::path::PathBuf::from("sub-a")], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 2 repositories"));
    assert!(output_str.contains("- 'umbrella': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
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
        true,           // include umbrella
        &[],            // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 2 repositories"));
    assert!(output_str.contains("- 'umbrella': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
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
        true,           // include umbrella
        &[],            // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 2 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("Pushing tags to remotes"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_push_reports_when_up_to_date(repo_sample: TestRepo) {
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    let remote_parent = repo_sample.repo_path.join("remotes");
    fs::create_dir_all(&remote_parent).unwrap();
    _run("git init --bare sub-a.git", &remote_parent).unwrap();
    let remote_path = remote_parent.join("sub-a.git");
    _run(
        &format!("git remote add origin {}", remote_path.display()),
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();
    _run(
        "git push -u origin main",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();

    // Seed the remote with an initial tag push.
    let mut first_output = Cursor::new(Vec::new());
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut first_output,
        Some("v1.0.0"),
        false,
        true,
        true,
        true,
        &[],
    )
    .unwrap();

    let mut output = Cursor::new(Vec::new());
    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        None,
        false,
        true,
        false,
        true,
        &[],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Listing tags in 2 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("Pushing tags to remotes"));
    assert!(output_str.contains("- 'sub-a': no tags to push"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
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
        true,  // include umbrella
        &[],   // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Listing tags in 1 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
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
        true,                                       // include umbrella
        &[std::path::PathBuf::from("nonexistent")], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Listing tags in 1 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn tag_no_repos_without_umbrella(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        None,
        false,
        false,
        false,
        false,
        &[],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("No repositories to tag"));
    assert!(!output_str.contains("- 'umbrella':"));
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
        true,           // include umbrella
        &[],            // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 3 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-b': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 3 repositories"));
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
        true,           // include umbrella
        &[
            std::path::PathBuf::from("sub-a"),
            std::path::PathBuf::from("sub-b"),
        ], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 3 repositories"));
    assert!(output_str.contains("- 'umbrella': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-b': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 3 repositories"));
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
        true,
        &[],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 2 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(!output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn tag_default_skips_configured_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::tag(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        None,
        false,
        false,
        false,
        true,
        &[],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Listing tags in 2 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(!output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b':"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
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
        true,
        &[std::path::PathBuf::from("sub-a")],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Creating tag 'v1.0.0' in 3 repositories"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("- 'sub-a': created tag 'v1.0.0'"));
    assert!(output_str.contains("- 'sub-b': created tag 'v1.0.0'"));
    assert!(output_str.contains("Successfully processed 3 repositories"));
}
