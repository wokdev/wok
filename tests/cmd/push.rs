use std::{fs, io::Cursor, path::PathBuf, process};

use rstest::*;

use git_wok::{cmd, config};

use super::*;

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn push_all_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the push command with --all
    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false, // set_upstream
        true,  // all
        None,  // branch
        true,  // include umbrella
        &[],   // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 2 repositories to branch 'main'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(
        output_str.contains("- 'sub-a':")
            && (output_str.contains("pushed to 'main'")
                || output_str.contains("already up to date")
                || output_str.contains("no remote configured, skipping"))
    );
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn push_specific_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the push command with specific repo
    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,                                // set_upstream
        false,                                // all
        None,                                 // branch
        true,                                 // include umbrella
        &[std::path::PathBuf::from("sub-a")], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 2 repositories to branch 'main'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(
        output_str.contains("- 'sub-a':")
            && (output_str.contains("pushed to 'main'")
                || output_str.contains("already up to date")
                || output_str.contains("no remote configured, skipping"))
    );
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn push_with_upstream_option(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the push command with --set-upstream
    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        true,  // set_upstream
        false, // all
        None,  // branch
        true,  // include umbrella
        &[],   // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 2 repositories to branch 'main'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(
        output_str.contains("- 'sub-a':")
            && (output_str.contains("pushed to 'main' and set upstream")
                || output_str.contains("already up to date")
                || output_str.contains("no remote configured, skipping"))
    );
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn push_with_branch_option(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the push command with --branch option
    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,           // set_upstream
        false,           // all
        Some("develop"), // branch
        true,            // include umbrella
        &[],             // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 2 repositories to branch 'develop'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn push_with_no_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the push command with no repos configured
    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false, // set_upstream
        false, // all
        None,  // branch
        true,  // include umbrella
        &[],   // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 1 repositories to branch 'main'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn push_nonexistent_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the push command with a non-existent repo
    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,                                      // set_upstream
        false,                                      // all
        None,                                       // branch
        true,                                       // include umbrella
        &[std::path::PathBuf::from("nonexistent")], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 1 repositories to branch 'main'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b.toml")))]
fn push_multiple_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the push command with multiple repos
    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false, // set_upstream
        true,  // all
        None,  // branch
        true,  // include umbrella
        &[],   // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 3 repositories to branch 'main'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b':"));
    assert!(output_str.contains("Successfully processed 3 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn push_all_skips_configured_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,
        true,
        None,
        true,
        &[],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 2 repositories to branch 'main'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(!output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b':"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn push_all_includes_explicit_repo_overrides_skip(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,
        true,
        None,
        true,
        &[std::path::PathBuf::from("sub-a")],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 3 repositories to branch 'main'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b':"));
    assert!(output_str.contains("Successfully processed 3 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b.toml")))]
fn push_specific_multiple_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the push command with specific multiple repos
    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false, // set_upstream
        false, // all
        None,  // branch
        true,  // include umbrella
        &[
            std::path::PathBuf::from("sub-a"),
            std::path::PathBuf::from("sub-b"),
        ], // repos
    )
    .unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 3 repositories to branch 'main'"));
    assert!(output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b':"));
    assert!(output_str.contains("Successfully processed 3 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn push_skips_umbrella_when_disabled(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,
        true,
        None,
        false,
        &[],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Pushing 1 repositories to branch 'main'"));
    assert!(!output_str.contains("- 'umbrella':"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn push_no_repos_without_umbrella(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,
        false,
        None,
        false,
        &[],
    )
    .unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("No repositories to push"));
    assert!(!output_str.contains("- 'umbrella':"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn push_lfs_objects_when_repo_uses_lfs(repo_sample: TestRepo) {
    if !has_git_lfs() {
        eprintln!("Skipping LFS test because git-lfs is not installed");
        return;
    }

    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();
    let remote_parent = repo_sample.repo_path.join("remotes");
    fs::create_dir_all(&remote_parent).unwrap();
    let remote_path = remote_parent.join("sub-a.git");

    _run("git init --bare sub-a.git", &remote_parent).unwrap();
    _run(
        &format!("git remote add origin {}", remote_path.display()),
        subrepo_path,
    )
    .unwrap();
    _run("git push -u origin main", subrepo_path).unwrap();

    _run("git lfs install --local", subrepo_path).unwrap();
    _run("git lfs track '*.bin'", subrepo_path).unwrap();
    fs::write(
        subrepo_path.join("artifact.bin"),
        "lfs payload from push test",
    )
    .unwrap();
    _run("git add .gitattributes artifact.bin", subrepo_path).unwrap();
    _run("git commit -m 'Add LFS artifact'", subrepo_path).unwrap();
    let blob_contents = run_checked("git show HEAD:artifact.bin", subrepo_path);
    assert!(
        blob_contents.contains("https://git-lfs.github.com/spec/v1"),
        "Expected artifact.bin to be committed as an LFS pointer",
    );

    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    cmd::push(
        &mut actual_config,
        &repo_sample.repo(),
        &mut output,
        false,
        false,
        Some("main"),
        false,
        &[std::path::PathBuf::from("sub-a")],
    )
    .unwrap();

    let verifier_path = remote_parent.join("verifier");
    run_checked(
        &format!(
            "git clone {} {}",
            remote_path.display(),
            verifier_path.display()
        ),
        &remote_parent,
    );
    run_checked("git lfs install --local", &verifier_path);
    let fetch_output = run_checked("git lfs fetch origin main", &verifier_path);
    assert!(
        !fetch_output.contains("Object does not exist on the server"),
        "Expected remote to have uploaded LFS objects, got: {fetch_output}",
    );
}

fn run_checked(cmd: &str, cwd: &PathBuf) -> String {
    let argv = shell_words::split(cmd).unwrap();
    let output = process::Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Command failed: `{}`\nstdout:\n{}\nstderr:\n{}",
        cmd,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}
