use std::env;
use std::process::Command;

use super::*;

/// Test that the implicit create syntax works with --sign flag
#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_implicit_create_with_sign_flag(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("tag")
        .arg("v1.0.0")
        .arg("--sign")
        .arg("--all")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to execute wok tag");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // May fail due to GPG not being configured, but the command should parse correctly
    // and attempt to create tags
    assert!(
        stdout.contains("Creating tag 'v1.0.0'") || stderr.contains("GPG"),
        "Expected tag creation attempt, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Test that the implicit create syntax works with --message flag
#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_implicit_create_with_message_flag(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("tag")
        .arg("v1.0.0")
        .arg("--message")
        .arg("Release v1.0.0")
        .arg("--all")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to execute wok tag");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Creating tag 'v1.0.0'"));
    assert!(stdout.contains("created tag 'v1.0.0'"));

    // Verify the tag was created as annotated tag with message
    let umbrella_repo = repo_sample.repo();
    let tag_obj = umbrella_repo
        .git_repo
        .revparse_single("refs/tags/v1.0.0")
        .expect("Tag should exist");
    let tag = tag_obj.as_tag().expect("Should be an annotated tag");
    assert_eq!(tag.message().unwrap().trim(), "Release v1.0.0");
}

/// Test that the implicit create syntax works with short -m flag
#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_implicit_create_with_message_short_flag(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("tag")
        .arg("v2.0.0")
        .arg("-m")
        .arg("Release v2.0.0")
        .arg("--all")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to execute wok tag");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Creating tag 'v2.0.0'"));
    assert!(stdout.contains("created tag 'v2.0.0'"));

    // Verify the tag was created as annotated tag with message
    let umbrella_repo = repo_sample.repo();
    let tag_obj = umbrella_repo
        .git_repo
        .revparse_single("refs/tags/v2.0.0")
        .expect("Tag should exist");
    let tag = tag_obj.as_tag().expect("Should be an annotated tag");
    assert_eq!(tag.message().unwrap().trim(), "Release v2.0.0");
}

/// Test that explicit create syntax also works with flags
#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_explicit_create_with_message_flag(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("tag")
        .arg("--all")
        .arg("create")
        .arg("v3.0.0")
        .arg("-m")
        .arg("Release v3.0.0")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to execute wok tag create");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    assert!(stdout.contains("Creating tag 'v3.0.0'"));
    assert!(stdout.contains("created tag 'v3.0.0'"));

    // Verify the tag was created as annotated tag with message
    let umbrella_repo = repo_sample.repo();
    let tag_obj = umbrella_repo
        .git_repo
        .revparse_single("refs/tags/v3.0.0")
        .expect("Tag should exist");
    let tag = tag_obj.as_tag().expect("Should be an annotated tag");
    assert_eq!(tag.message().unwrap().trim(), "Release v3.0.0");
}

/// Test that list subcommand works
#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_list_subcommand(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    // First create a tag
    let _ = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("tag")
        .arg("--all")
        .arg("create")
        .arg("v1.0.0")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to create tag");

    // Now list tags
    let output = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("tag")
        .arg("--all")
        .arg("list")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to execute wok tag list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    assert!(stdout.contains("✅ umbrella [v1.0.0]: v1.0.0"));
    assert!(stdout.contains("✅ sub-a [v1.0.0]: v1.0.0"));
}

/// Test that implicit list works (no subcommand, no args)
#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn tag_implicit_list(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    // First create a tag
    let _ = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("tag")
        .arg("--all")
        .arg("create")
        .arg("v1.0.0")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to create tag");

    // Now list tags using implicit syntax
    let output = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("tag")
        .arg("--all")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to execute wok tag");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    assert!(
        stdout.contains("✅ umbrella [v1.0.0]: v1.0.0"),
        "Expected umbrella row in output: {}",
        stdout
    );
    assert!(
        stdout.contains("✅ sub-a [v1.0.0]: v1.0.0"),
        "Expected sub-a row in output: {}",
        stdout
    );
    assert!(
        stdout.contains("v1.0.0"),
        "Expected 'v1.0.0' in output: {}",
        stdout
    );
}

/// Test that help for tag create shows sign and message options
#[test]
fn tag_create_help_shows_options() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("tag")
        .arg("create")
        .arg("--help")
        .output()
        .expect("Failed to execute wok tag create --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--sign") || stdout.contains("-s"));
    assert!(stdout.contains("--message") || stdout.contains("-m"));
}

/// Test that help for bare tag command mentions subcommands
#[test]
fn tag_help_mentions_subcommands() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("tag")
        .arg("--help")
        .output()
        .expect("Failed to execute wok tag --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("create"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("push"));
    // The hidden flags should not appear in the main help
    assert!(!stdout.contains("-s") || !stdout.contains("Sign the tag"));
}
