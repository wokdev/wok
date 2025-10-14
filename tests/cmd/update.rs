use std::{fs, io::Cursor};

use rstest::*;

use wok_dev::{cmd, config};

use super::*;

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
