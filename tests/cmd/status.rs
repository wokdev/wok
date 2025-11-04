use std::io::Cursor;

use pretty_assertions::assert_eq;
use rstest::*;

use git_wok::{cmd, config};

use super::*;

#[rstest(repo_sample(vec![], Some("empty.toml")))]
fn no_submodules_no_changes(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output, false).unwrap();

    assert_eq!(
        String::from_utf8_lossy(output.get_ref()),
        format!("On branch '{}', all clean\n", "main")
    );
}

fn _no_submodules_with_changes() {}
#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn with_submodules_branch_matches_no_changes(repo_sample: TestRepo) {
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output, false).unwrap();

    let expected =
        "On branch 'main', all clean\n- 'sub-a' is on branch 'main', all clean\n";
    assert_eq!(String::from_utf8_lossy(output.get_ref()), expected);
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn status_shows_up_to_date_with_remote(repo_sample: TestRepo) {
    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();

    // Setup remote
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

    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output, false).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("up to date with 'origin/main'"),
        "Expected 'up to date with origin/main' in output: {output_str}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn status_shows_ahead_of_remote(repo_sample: TestRepo) {
    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();

    // Setup remote
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

    // Create local commit without pushing
    fs::write(subrepo_path.join("LOCAL.md"), "local change").unwrap();
    _run("git add LOCAL.md", subrepo_path).unwrap();
    _run("git commit -m 'local commit'", subrepo_path).unwrap();

    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output, false).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("ahead of 'origin/main' by 1 commit"),
        "Expected 'ahead of origin/main by 1 commit' in output: {output_str}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn status_shows_ahead_of_remote_multiple_commits(repo_sample: TestRepo) {
    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();

    // Setup remote
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

    // Create two local commits without pushing
    fs::write(subrepo_path.join("LOCAL1.md"), "local change 1").unwrap();
    _run("git add LOCAL1.md", subrepo_path).unwrap();
    _run("git commit -m 'local commit 1'", subrepo_path).unwrap();
    fs::write(subrepo_path.join("LOCAL2.md"), "local change 2").unwrap();
    _run("git add LOCAL2.md", subrepo_path).unwrap();
    _run("git commit -m 'local commit 2'", subrepo_path).unwrap();

    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output, false).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("ahead of 'origin/main' by 2 commits"),
        "Expected 'ahead of origin/main by 2 commits' in output: {output_str}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn status_shows_behind_remote(repo_sample: TestRepo) {
    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();

    // Setup remote
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

    // Create remote commit via contributor clone
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

    // Fetch to update remote refs, then check status
    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output, true).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("behind 'origin/main' by 1 commit"),
        "Expected 'behind origin/main by 1 commit' in output: {output_str}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn status_shows_diverged(repo_sample: TestRepo) {
    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();

    // Setup remote
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

    // Create local commit
    fs::write(subrepo_path.join("LOCAL.md"), "local change").unwrap();
    _run("git add LOCAL.md", subrepo_path).unwrap();
    _run("git commit -m 'local commit'", subrepo_path).unwrap();

    // Create remote commit via contributor clone
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

    // Fetch to update remote refs, then check status
    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output, true).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("diverged from 'origin/main' (1 ahead, 1 behind)"),
        "Expected 'diverged from origin/main (1 ahead, 1 behind)' in output: {output_str}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn status_handles_no_tracking_branch(repo_sample: TestRepo) {
    // No remote configured, should show no remote status
    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output, false).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    // Should not show any remote status
    assert!(
        !output_str.contains("origin/main")
            && !output_str.contains("ahead")
            && !output_str.contains("behind"),
        "Expected no remote status in output: {output_str}"
    );
    assert!(
        output_str.contains("On branch 'main', all clean\n"),
        "Expected basic status line: {output_str}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn status_with_fetch_flag_updates_remote_refs(repo_sample: TestRepo) {
    let subrepo_path = repo_sample.subrepo_paths.get("sub-a").unwrap();

    // Setup remote
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

    // First status check - should be up to date
    let mut output1 = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output1, false).unwrap();

    let output_str1 = String::from_utf8_lossy(output1.get_ref());
    assert!(
        output_str1.contains("up to date with 'origin/main'"),
        "Expected up to date initially: {output_str1}"
    );

    // Create remote commit via contributor clone
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

    // Status without fetch - should still show up to date (stale remote refs)
    let mut output2 = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output2, false).unwrap();

    let output_str2 = String::from_utf8_lossy(output2.get_ref());
    assert!(
        output_str2.contains("up to date with 'origin/main'"),
        "Expected up to date without fetch: {output_str2}"
    );

    // Status with fetch - should now show behind
    let mut output3 = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();
    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output3, true).unwrap();

    let output_str3 = String::from_utf8_lossy(output3.get_ref());
    assert!(
        output_str3.contains("behind 'origin/main' by 1 commit"),
        "Expected behind after fetch: {output_str3}"
    );
}

#[rstest(repo_sample(vec!["sub-a"], Some("a.toml")))]
fn status_shows_umbrella_remote_status(repo_sample: TestRepo) {
    // Setup remote for umbrella repo
    _run("git add .", &repo_sample.repo_path).unwrap();
    _run("git commit -m baseline", &repo_sample.repo_path).unwrap();

    let remote_parent = repo_sample.repo_path.join("remotes");
    fs::create_dir_all(&remote_parent).unwrap();
    let remote_path = remote_parent.join("umbrella.git");

    _run("git init --bare umbrella.git", &remote_parent).unwrap();
    _run(
        &format!("git remote add origin {}", remote_path.display()),
        &repo_sample.repo_path,
    )
    .unwrap();
    _run("git push -u origin main", &repo_sample.repo_path).unwrap();

    // Create local commit in umbrella without pushing
    fs::write(repo_sample.repo_path.join("README.md"), "umbrella change").unwrap();
    _run("git add README.md", &repo_sample.repo_path).unwrap();
    _run("git commit -m 'umbrella commit'", &repo_sample.repo_path).unwrap();

    let mut output = Cursor::new(Vec::new());
    let mut actual_config = config::Config::load(&repo_sample.config_path()).unwrap();

    cmd::status(&mut actual_config, &repo_sample.repo(), &mut output, false).unwrap();

    let output_str = String::from_utf8_lossy(output.get_ref());
    assert!(
        output_str.contains("On branch 'main'")
            && output_str.contains("ahead of 'origin/main' by 1 commit"),
        "Expected umbrella to show ahead status: {output_str}"
    );
}

fn _with_submodules_branch_matches_with_changes() {}
fn _with_submodules_branch_doesnt_match_no_changes() {}
fn _with_submodules_branch_doesnt_match_with_changes() {}
