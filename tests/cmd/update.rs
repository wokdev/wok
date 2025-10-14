use std::{fs, io::Cursor};

use rstest::*;

use wok_dev::{cmd, config};

use super::*;

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn update_pulls_tracking_branch(repo_sample: TestRepo) {
    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();

    _run("git add .", &repo_sample.repo_path).unwrap();
    _run("git commit -m baseline", &repo_sample.repo_path).unwrap();

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

    let contributor_path = remote_parent.join("contributor");
    _run(
        &format!(
            "git clone {} {}",
            remote_path.display(),
            contributor_path.display()
        ),
        &remote_parent,
    )
    .unwrap();
    _run("git config user.email 'test@localhost'", &contributor_path).unwrap();
    _run("git config user.name 'Test User'", &contributor_path).unwrap();
    fs::write(contributor_path.join("UPSTREAM.md"), "upstream change").unwrap();
    _run("git add UPSTREAM.md", &contributor_path).unwrap();
    _run("git commit -m upstream", &contributor_path).unwrap();
    _run("git push", &contributor_path).unwrap();

    let local_before = _run("git rev-parse HEAD", subrepo_path).unwrap();
    let umbrella_head_before =
        _run("git rev-parse HEAD", &repo_sample.repo_path).unwrap();

    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let umbrella = repo_sample.repo();

    cmd::update(&mut actual_config, &umbrella, &mut output, false).unwrap();

    let local_after = _run("git rev-parse HEAD", subrepo_path).unwrap();
    let remote_tip = _run("git rev-parse origin/main", subrepo_path).unwrap();
    assert_ne!(local_before.trim(), local_after.trim());
    assert_eq!(local_after.trim(), remote_tip.trim());

    let umbrella_head_after =
        _run("git rev-parse HEAD", &repo_sample.repo_path).unwrap();
    assert_ne!(umbrella_head_before.trim(), umbrella_head_after.trim());

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("- 'sub-a': fast-forwarded 'main'"),
        "Output: {output_str}"
    );
    assert!(
        output_str.contains("Updated submodule state committed"),
        "Output: {output_str}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn update_submodules(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    _run("git add .", &repo_sample.repo_path).unwrap();
    _run("git commit -m baseline", &repo_sample.repo_path).unwrap();
    let status = _run("git status --short", &repo_sample.repo_path).unwrap();
    assert!(
        status.trim().is_empty(),
        "Expected clean repo before update; status: {status}"
    );
    let umbrella = repo_sample.repo();

    let head_before = umbrella
        .git_repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id();

    // Run the update command
    cmd::update(&mut actual_config, &umbrella, &mut output, false).unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("Updating submodules..."),
        "Output: {output_str}"
    );
    assert!(
        output_str.contains("- 'sub-a': already up to date on 'main'"),
        "Output: {output_str}"
    );
    assert!(
        output_str.contains("No submodule updates detected; nothing to commit"),
        "Output: {output_str}"
    );
    assert!(
        !output_str.contains("Updated submodule state committed"),
        "Output: {output_str}"
    );

    let head_after = umbrella
        .git_repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id();
    assert_eq!(head_before, head_after);
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn update_with_no_submodules(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let umbrella = repo_sample.repo();

    let head_before = umbrella
        .git_repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id();

    // Run the update command with no submodules
    cmd::update(&mut actual_config, &umbrella, &mut output, false).unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("Updating submodules..."),
        "Output: {output_str}"
    );
    assert!(
        output_str.contains("No submodule updates detected; nothing to commit"),
        "Output: {output_str}"
    );
    assert!(
        !output_str.contains("Updated submodule state committed"),
        "Output: {output_str}"
    );

    let head_after = umbrella
        .git_repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id();
    assert_eq!(head_before, head_after);
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn update_skips_configured_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let umbrella = repo_sample.repo();

    cmd::update(&mut actual_config, &umbrella, &mut output, false).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("Updating submodules..."),
        "Output: {output_str}"
    );
    assert!(!output_str.contains("- 'sub-a':"), "Output: {output_str}");
    assert!(output_str.contains("- 'sub-b':"), "Output: {output_str}");
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn update_with_no_commit_flag_skips_commit(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Stage a change in the umbrella repo so update would normally commit
    fs::write(repo_sample.repo_path.join("README.md"), "pending change").unwrap();
    _run("git add README.md", &repo_sample.repo_path).unwrap();

    let umbrella = repo_sample.repo();
    let head_before = umbrella
        .git_repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id();

    cmd::update(&mut actual_config, &umbrella, &mut output, true).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("Updating submodules..."),
        "Output: {output_str}"
    );
    assert!(
        output_str.contains(
            "Changes staged; commit skipped because --no-commit was provided",
        ),
        "Output: {output_str}"
    );
    assert!(
        !output_str.contains("Updated submodule state committed"),
        "Output: {output_str}"
    );

    let head_after = umbrella
        .git_repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id();
    assert_eq!(head_before, head_after);
}
