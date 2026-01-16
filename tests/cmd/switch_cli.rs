use std::env;
use std::fs;
use std::process::Command;

use rstest::*;

use super::*;

/// Ensure `wok switch -b <BRANCH>` works as a short alias for `--branch`.
#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_with_branch_short_flag(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = env::var("CARGO_BIN_EXE_wok")
        .unwrap_or_else(|_| format!("{}/target/debug/wok", cargo_manifest_dir));

    let sub_a_path = &repo_sample.subrepo_paths["sub-a"];
    _run("git switch -c develop", sub_a_path).unwrap();
    _run("git switch main", sub_a_path).unwrap();
    _run("git add wok.toml", repo_sample.repo_path()).unwrap();
    _run(
        "git commit --allow-empty -m 'Add base wokfile'",
        repo_sample.repo_path(),
    )
    .unwrap();

    let output = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("switch")
        .arg("-b")
        .arg("develop")
        .arg("--all")
        .arg("-c")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to execute wok switch");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(stdout.contains("Switching 1 repositories for umbrella branch 'develop'"));
    assert!(stdout.contains("Successfully switched and locked 1 repositories"));
}

/// Ensure umbrella branch is switched first and wokfile from that branch is used.
#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b.toml")))]
fn switch_uses_target_branch_wokfile(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = env::var("CARGO_BIN_EXE_wok")
        .unwrap_or_else(|_| format!("{}/target/debug/wok", cargo_manifest_dir));

    let other_config = r#"version = "1.0"

[[repo]]
path = "sub-a"
head = "other"

[[repo]]
path = "sub-b"
head = "main"
"#;

    _run("git add wok.toml", repo_sample.repo_path()).unwrap();
    _run(
        "git commit --allow-empty -m 'Add base wokfile'",
        repo_sample.repo_path(),
    )
    .unwrap();

    _run("git branch -f other", repo_sample.repo_path()).unwrap();
    _run("git switch other", repo_sample.repo_path()).unwrap();
    fs::write(repo_sample.config_path(), other_config).unwrap();
    _run("git add wok.toml", repo_sample.repo_path()).unwrap();
    _run(
        "git commit --allow-empty -m 'Update wokfile for other'",
        repo_sample.repo_path(),
    )
    .unwrap();
    let other_contents = fs::read_to_string(repo_sample.config_path()).unwrap();
    assert!(
        other_contents.contains("head = \"other\""),
        "other branch wokfile missing expected head: {}",
        other_contents
    );
    let other_commit_contents =
        _run("git show HEAD:wok.toml", repo_sample.repo_path()).unwrap();
    assert!(
        other_commit_contents.contains("head = \"other\""),
        "other branch commit missing expected head: {}",
        other_commit_contents
    );
    _run("git switch main", repo_sample.repo_path()).unwrap();

    _run("git switch main", &repo_sample.subrepo_paths["sub-a"]).unwrap();
    _run("git switch main", &repo_sample.subrepo_paths["sub-b"]).unwrap();
    _run("git switch -c other", &repo_sample.subrepo_paths["sub-a"]).unwrap();
    _run("git switch main", &repo_sample.subrepo_paths["sub-a"]).unwrap();

    let output = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("switch")
        .arg("-b")
        .arg("other")
        .current_dir(repo_sample.repo_path())
        .output()
        .expect("Failed to execute wok switch");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    let umbrella_branch =
        _run("git branch --show-current", repo_sample.repo_path()).unwrap();
    assert_eq!(umbrella_branch.trim(), "other");

    let sub_a_branch = _run(
        "git branch --show-current",
        &repo_sample.subrepo_paths["sub-a"],
    )
    .unwrap();
    let sub_b_branch = _run(
        "git branch --show-current",
        &repo_sample.subrepo_paths["sub-b"],
    )
    .unwrap();
    let after_contents = fs::read_to_string(repo_sample.config_path()).unwrap();
    assert!(
        after_contents.contains("head = \"other\""),
        "after switch wokfile mismatch: {after_contents}"
    );
    let head_contents =
        _run("git show HEAD:wok.toml", repo_sample.repo_path()).unwrap();
    assert!(
        head_contents.contains("head = \"other\""),
        "HEAD wokfile mismatch: {head_contents}"
    );

    assert_eq!(
        sub_a_branch.trim(),
        "other",
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert_eq!(sub_b_branch.trim(), "main");
}
