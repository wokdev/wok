use std::env;
use std::process::Command;

use rstest::*;

use super::*;

/// Ensure `wok switch -b <BRANCH>` works as a short alias for `--branch`.
#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_with_branch_short_flag(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let sub_a_path = &repo_sample.subrepo_paths["sub-a"];
    _run("git switch -c develop", sub_a_path).unwrap();
    _run("git switch main", sub_a_path).unwrap();

    let output = Command::new(&wok_binary)
        .arg("-f")
        .arg(repo_sample.config_path())
        .arg("switch")
        .arg("-b")
        .arg("develop")
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
    assert!(stdout.contains("Switching 1 repositories to branch 'develop'"));
    assert!(stdout.contains("Successfully switched and locked 1 repositories"));
}
