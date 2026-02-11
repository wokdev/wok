use std::{env, fs, io::Cursor, path::Path};

use rstest::*;

use git_wok::{cmd, config};
use git2::Repository;

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

    cmd::update(&mut actual_config, &umbrella, &mut output, false, true).unwrap();

    let local_after = _run("git rev-parse HEAD", subrepo_path).unwrap();
    let remote_tip = _run("git rev-parse origin/main", subrepo_path).unwrap();
    assert_ne!(local_before.trim(), local_after.trim());
    assert_eq!(local_after.trim(), remote_tip.trim());

    let umbrella_head_after =
        _run("git rev-parse HEAD", &repo_sample.repo_path).unwrap();
    assert_ne!(umbrella_head_before.trim(), umbrella_head_after.trim());

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("Updating repositories..."),
        "Output: {output_str}"
    );
    assert!(
        output_str.contains("- 'sub-a': fast-forwarded 'main'"),
        "Output: {output_str}"
    );
    assert!(output_str.contains("- 'umbrella':"), "Output: {output_str}");
    assert!(
        output_str.contains("Updated submodule state committed"),
        "Output: {output_str}"
    );
}

#[rstest(repo_sample(vec![], None))]
fn update_switches_detached_submodule_to_configured_branch(repo_sample: TestRepo) {
    let umbrella_path = repo_sample.repo_path();
    let remote_parent = umbrella_path.join("remotes");
    fs::create_dir_all(&remote_parent).unwrap();

    let source_path = remote_parent.join("source");
    fs::create_dir_all(&source_path).unwrap();
    _run("git init -b main", &source_path).unwrap();
    _run("git config user.email 'test@localhost'", &source_path).unwrap();
    _run("git config user.name 'Test User'", &source_path).unwrap();
    fs::write(source_path.join("README.md"), "initial").unwrap();
    _run("git add README.md", &source_path).unwrap();
    _run("git commit -m initial", &source_path).unwrap();

    let remote_path = remote_parent.join("sub-a.git");
    _run("git init --bare sub-a.git", &remote_parent).unwrap();
    _run(
        &format!("git remote add origin {}", remote_path.display()),
        &source_path,
    )
    .unwrap();
    _run("git push -u origin main", &source_path).unwrap();

    _run(
        &format!(
            "git -c protocol.file.allow=always submodule add {} sub-a",
            remote_path.display()
        ),
        umbrella_path,
    )
    .unwrap();
    _run(
        "git -c protocol.file.allow=always submodule update --init",
        umbrella_path,
    )
    .unwrap();

    let mut actual_config = config::Config::new();
    actual_config.add_repo(Path::new("sub-a"), "main");
    actual_config.save(&repo_sample.config_path()).unwrap();

    let subrepo_path = umbrella_path.join("sub-a");
    _run("git checkout --detach", &subrepo_path).unwrap();
    let subrepo_git = Repository::open(&subrepo_path).unwrap();
    assert!(
        subrepo_git.head_detached().unwrap(),
        "Expected detached HEAD before update"
    );

    let mut output = Cursor::new(Vec::new());
    let umbrella = repo_sample.repo();
    cmd::update(&mut actual_config, &umbrella, &mut output, false, true).unwrap();

    let subrepo_git = Repository::open(&subrepo_path).unwrap();
    assert!(
        !subrepo_git.head_detached().unwrap(),
        "Expected attached HEAD after update"
    );
    let head_name = subrepo_git.head().unwrap().shorthand().unwrap().to_string();
    assert_eq!(head_name, "main");

    let upstream = _run(
        "git rev-parse --abbrev-ref --symbolic-full-name @{u}",
        &subrepo_path,
    )
    .unwrap();
    assert_eq!(upstream.trim(), "origin/main");
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
    cmd::update(&mut actual_config, &umbrella, &mut output, false, true).unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("Updating repositories..."),
        "Output: {output_str}"
    );
    assert!(
        output_str.contains("- 'umbrella': already up to date"),
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
    cmd::update(&mut actual_config, &umbrella, &mut output, false, true).unwrap();

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("Updating repositories..."),
        "Output: {output_str}"
    );
    assert!(
        output_str.contains("- 'umbrella': already up to date"),
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

    cmd::update(&mut actual_config, &umbrella, &mut output, false, true).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("Updating repositories..."),
        "Output: {output_str}"
    );
    assert!(output_str.contains("- 'umbrella':"), "Output: {output_str}");
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

    cmd::update(&mut actual_config, &umbrella, &mut output, true, true).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("Updating repositories..."),
        "Output: {output_str}"
    );
    assert!(output_str.contains("- 'umbrella':"), "Output: {output_str}");
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

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn update_respects_rebase_config(repo_sample: TestRepo) {
    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();

    // Configure the subrepo to use rebase
    _run("git config pull.rebase true", subrepo_path).unwrap();

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

    // Create a local commit
    fs::write(subrepo_path.join("LOCAL.md"), "local change").unwrap();
    _run("git add LOCAL.md", subrepo_path).unwrap();
    _run("git commit -m 'local commit'", subrepo_path).unwrap();

    // Create an upstream commit in a contributor clone
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

    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let umbrella = repo_sample.repo();

    cmd::update(&mut actual_config, &umbrella, &mut output, false, true).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    // Should say "rebased" not "merged"
    assert!(
        output_str.contains("- 'sub-a': rebased 'main'"),
        "Output: {output_str}"
    );
    assert!(output_str.contains("- 'umbrella':"), "Output: {output_str}");
    assert!(
        output_str.contains("Updated submodule state committed"),
        "Output: {output_str}"
    );

    // Verify that the history is linear (rebase), not a merge commit
    let log_output = _run("git log --oneline --graph", subrepo_path).unwrap();
    assert!(
        !log_output.contains("Merge"),
        "Expected linear history from rebase, got: {log_output}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn update_uses_merge_by_default(repo_sample: TestRepo) {
    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();

    // Explicitly set to merge (or leave default)
    _run("git config pull.rebase false", subrepo_path).unwrap();

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

    // Create a local commit
    fs::write(subrepo_path.join("LOCAL.md"), "local change").unwrap();
    _run("git add LOCAL.md", subrepo_path).unwrap();
    _run("git commit -m 'local commit'", subrepo_path).unwrap();

    // Create an upstream commit
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

    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let umbrella = repo_sample.repo();

    cmd::update(&mut actual_config, &umbrella, &mut output, false, true).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    // Should say "merged" when pull.rebase is false
    assert!(
        output_str.contains("- 'sub-a': merged 'main'"),
        "Output: {output_str}"
    );
    assert!(output_str.contains("- 'umbrella':"), "Output: {output_str}");
    assert!(
        output_str.contains("Updated submodule state committed"),
        "Output: {output_str}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn update_skips_umbrella_when_disabled(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let umbrella = repo_sample.repo();

    cmd::update(&mut actual_config, &umbrella, &mut output, false, false).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("- 'sub-a':"), "Output: {output_str}");
    assert!(
        !output_str.contains("- 'umbrella':"),
        "Output: {output_str}"
    );
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn update_without_umbrella_when_no_submodules(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let umbrella = repo_sample.repo();

    cmd::update(&mut actual_config, &umbrella, &mut output, false, false).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("No submodule updates detected; nothing to commit"),
        "Output: {output_str}"
    );
    assert!(
        !output_str.contains("- 'umbrella':"),
        "Output: {output_str}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn update_pulls_lfs_objects_when_repo_uses_lfs(repo_sample: TestRepo) {
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

    _run("git lfs install --local", subrepo_path).unwrap();
    _run("git lfs track '*.bin'", subrepo_path).unwrap();
    fs::write(subrepo_path.join("seed.bin"), "initial lfs payload").unwrap();
    _run("git add .gitattributes seed.bin", subrepo_path).unwrap();
    _run("git commit -m 'seed lfs'", subrepo_path).unwrap();
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
    _run("git lfs install --local", &contributor_path).unwrap();

    let expected_content = "upstream lfs payload";
    fs::write(contributor_path.join("UPSTREAM.bin"), expected_content).unwrap();
    _run("git add UPSTREAM.bin", &contributor_path).unwrap();
    _run("git commit -m upstream-lfs", &contributor_path).unwrap();
    _run("git push", &contributor_path).unwrap();

    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let umbrella = repo_sample.repo();
    cmd::update(&mut actual_config, &umbrella, &mut output, false, true).unwrap();

    let updated_file = fs::read_to_string(subrepo_path.join("UPSTREAM.bin")).unwrap();
    assert_eq!(updated_file, expected_content);
    assert!(
        !updated_file.contains("https://git-lfs.github.com/spec/v1"),
        "Expected materialized LFS object, got pointer text",
    );
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn update_initializes_newly_configured_missing_submodule(repo_sample: TestRepo) {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = env::var("CARGO_BIN_EXE_wok")
        .unwrap_or_else(|_| format!("{}/target/debug/wok", cargo_manifest_dir));

    let source_umbrella_path = repo_sample.repo_path();
    let remote_parent = source_umbrella_path.join("remotes");
    fs::create_dir_all(&remote_parent).unwrap();

    let source_path = remote_parent.join("source-sub-b");
    fs::create_dir_all(&source_path).unwrap();
    _run("git init -b main", &source_path).unwrap();
    _run("git config user.email 'test@localhost'", &source_path).unwrap();
    _run("git config user.name 'Test User'", &source_path).unwrap();
    fs::write(source_path.join("README.md"), "sub-b initial").unwrap();
    _run("git add README.md", &source_path).unwrap();
    _run("git commit -m initial", &source_path).unwrap();

    let remote_path = remote_parent.join("sub-b.git");
    _run("git init --bare sub-b.git", &remote_parent).unwrap();
    _run(
        &format!("git remote add origin {}", remote_path.display()),
        &source_path,
    )
    .unwrap();
    _run("git push -u origin main", &source_path).unwrap();

    _run(
        &format!(
            "git -c protocol.file.allow=always submodule add {} sub-b",
            remote_path.display()
        ),
        source_umbrella_path,
    )
    .unwrap();

    _run("git add .gitmodules sub-b", source_umbrella_path).unwrap();
    _run("git commit -m 'add sub-b submodule'", source_umbrella_path).unwrap();

    let mut wok_config = config::Config::load(&repo_sample.config_path()).unwrap();
    wok_config.add_repo(Path::new("sub-b"), "main");
    wok_config.save(&repo_sample.config_path()).unwrap();
    _run("git add wok.toml", source_umbrella_path).unwrap();
    _run(
        "git commit -m 'configure sub-b for update'",
        source_umbrella_path,
    )
    .unwrap();

    let clone_path = remote_parent.join("consumer");
    _run(
        &format!(
            "git clone {} {}",
            source_umbrella_path.display(),
            clone_path.display()
        ),
        &remote_parent,
    )
    .unwrap();
    _run("git config user.email 'test@localhost'", &clone_path).unwrap();
    _run("git config user.name 'Test User'", &clone_path).unwrap();

    let clone_config_path = clone_path.join("wok.toml");
    let sub_b_abs_path = clone_path.join("sub-b");

    let output = std::process::Command::new(&wok_binary)
        .arg("-f")
        .arg(&clone_config_path)
        .arg("update")
        .arg("--no-commit")
        .current_dir(&clone_path)
        .output()
        .expect("Failed to execute wok update");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("- 'sub-b':"),
        "Expected sub-b to be processed. stdout: {stdout}"
    );
    assert!(
        Repository::open(&sub_b_abs_path).is_ok(),
        "Expected sub-b to be initialized at `{}`",
        sub_b_abs_path.display()
    );
}
