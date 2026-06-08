use std::{fs, io::Cursor, path::Path};

use rstest::*;

use git_wok::{cmd, config};

use super::*;

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_all_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with --all
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        false, // create
        true,  // all
        false, // dirty
        "main",
        &[], // repos
    )
    .unwrap();

    assert!(!config_changed);
    let repo_entry = actual_config
        .repos
        .iter()
        .find(|r| r.path == Path::new("sub-a"))
        .unwrap();
    assert_eq!(repo_entry.head, "main");

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories for umbrella branch 'main'"));
    assert!(
        output_str.contains("- 'sub-a':")
            && (output_str.contains("switched to 'main'")
                || output_str.contains("already on 'main'"))
    );
    assert!(output_str.contains("No workspace changes detected; skipping lock"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn switch_all_skips_configured_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        false,
        true,
        false, // dirty
        "main",
        &[],
    )
    .unwrap();

    assert!(!config_changed);
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories for umbrella branch 'main'"));
    assert!(!output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b':"));
    assert!(output_str.contains("No workspace changes detected; skipping lock"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a", "sub-b"], Some("a-b-skip.toml")))]
fn switch_all_includes_explicit_repo_overrides_skip(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        false,
        true,
        false, // dirty
        "main",
        &[std::path::PathBuf::from("sub-a")],
    )
    .unwrap();

    assert!(!config_changed);
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 2 repositories for umbrella branch 'main'"));
    assert!(output_str.contains("- 'sub-a':"));
    assert!(output_str.contains("- 'sub-b':"));
    assert!(output_str.contains("No workspace changes detected; skipping lock"));
    assert!(output_str.contains("Successfully processed 2 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_specific_repo(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with specific repo
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        false, // create
        false, // all
        false, // dirty
        "main",
        &[std::path::PathBuf::from("sub-a")], // repos
    )
    .unwrap();

    assert!(!config_changed);
    let repo_entry = actual_config
        .repos
        .iter()
        .find(|r| r.path == Path::new("sub-a"))
        .unwrap();
    assert_eq!(repo_entry.head, "main");

    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories for umbrella branch 'main'"));
    assert!(
        output_str.contains("- 'sub-a':")
            && (output_str.contains("switched to 'main'")
                || output_str.contains("already on 'main'"))
    );
    assert!(output_str.contains("No workspace changes detected; skipping lock"));
    assert!(output_str.contains("Successfully processed 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_with_create_option(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with --create and a new branch name
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        true,  // create
        true,  // all
        false, // dirty
        "feature-branch",
        &[], // repos
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
    assert!(
        output_str
            .contains("Switching 1 repositories for umbrella branch 'feature-branch'")
    );
    assert!(output_str.contains("Locking workspace state"));
    assert!(output_str.contains("- 'sub-a': created and switched to 'feature-branch'"));
    assert!(output_str.contains("Successfully switched and locked 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_with_branch_option(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    _run("git switch -c develop", &repo_sample.subrepo_paths["sub-a"]).unwrap();
    _run("git switch main", &repo_sample.subrepo_paths["sub-a"]).unwrap();

    // Run the switch command with --branch option
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        false, // create
        true,  // all
        false, // dirty
        "develop",
        &[], // repos
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
    assert!(
        output_str.contains("Switching 1 repositories for umbrella branch 'develop'")
    );
    assert!(output_str.contains("Locking workspace state"));
    assert!(output_str.contains("Successfully switched and locked 1 repositories"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_refreshes_worktree_when_already_on_branch(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let subrepo_path = &repo_sample.subrepo_paths["sub-a"];
    let file_path = subrepo_path.join("message.txt");

    fs::write(&file_path, "v1\n").unwrap();
    _run("git add message.txt", subrepo_path).unwrap();
    _run("git commit -m 'Add v1'", subrepo_path).unwrap();

    _run("git switch -c temp", subrepo_path).unwrap();
    fs::write(&file_path, "v2\n").unwrap();
    _run("git add message.txt", subrepo_path).unwrap();
    _run("git commit -m 'Add v2'", subrepo_path).unwrap();
    let v2_commit = _run("git rev-parse HEAD", subrepo_path).unwrap();

    _run("git switch main", subrepo_path).unwrap();
    _run(
        &format!("git update-ref refs/heads/main {}", v2_commit.trim()),
        subrepo_path,
    )
    .unwrap();

    let before_contents = fs::read_to_string(&file_path).unwrap();
    assert_eq!(before_contents, "v1\n");

    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        false, // create
        false, // all
        false, // dirty
        "main",
        &[], // repos
    )
    .unwrap();

    let after_contents = fs::read_to_string(&file_path).unwrap();
    assert_eq!(after_contents, "v2\n");
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_updates_file_contents_for_target_branch(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    let subrepo_path = &repo_sample.subrepo_paths["sub-a"];
    let file_path = subrepo_path.join("message.txt");

    fs::write(&file_path, "main\n").unwrap();
    _run("git add message.txt", subrepo_path).unwrap();
    _run("git commit -m 'Add main content'", subrepo_path).unwrap();

    _run("git switch -c feature", subrepo_path).unwrap();
    fs::write(&file_path, "feature\n").unwrap();
    _run("git add message.txt", subrepo_path).unwrap();
    _run("git commit -m 'Add feature content'", subrepo_path).unwrap();
    _run("git switch main", subrepo_path).unwrap();

    let before_contents = fs::read_to_string(&file_path).unwrap();
    assert_eq!(before_contents, "main\n");

    cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        false, // create
        true,  // all
        false, // dirty
        "feature",
        &[], // repos
    )
    .unwrap();

    let after_contents = fs::read_to_string(&file_path).unwrap();
    assert_eq!(after_contents, "feature\n");
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_all_repos_moves_repo_when_cached_head_matches_target(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    _run("git switch -c test", &repo_sample.repo_path).unwrap();
    _run("git switch -c test", &repo_sample.subrepo_paths["sub-a"]).unwrap();
    _run("git switch main", &repo_sample.subrepo_paths["sub-a"]).unwrap();

    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        false, // create
        true,  // all
        false, // dirty
        "test",
        &[], // repos
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
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Locking workspace state"));
    assert!(output_str.contains("Successfully switched and locked 1 repositories"));
}

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn switch_with_no_repos(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    // Run the switch command with no repos configured
    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        false, // create
        false, // all
        false, // dirty
        "main",
        &[], // repos
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
        &repo_sample.config_path(),
        &mut output,
        false, // create
        false, // all
        false, // dirty
        "main",
        &[std::path::PathBuf::from("nonexistent")], // repos
    )
    .unwrap();

    assert!(!config_changed);
    // Check the output
    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(output_str.contains("Switching 1 repositories for umbrella branch 'main'"));
    assert!(output_str.contains("- 'sub-a':"));
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn switch_commits_wokfile_with_submodule_state(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    let config_changed = cmd::switch(
        &mut actual_config,
        &repo_sample.repo(),
        &repo_sample.config_path(),
        &mut output,
        true,  // create
        true,  // all
        false, // dirty
        "feature-branch",
        &[],
    )
    .unwrap();

    assert!(config_changed);

    let committed_files = _run(
        "git show --name-only --pretty=format:",
        repo_sample.repo_path(),
    )
    .unwrap();
    assert!(committed_files.contains("wok.toml"));
    assert!(committed_files.contains("sub-a"));
}
