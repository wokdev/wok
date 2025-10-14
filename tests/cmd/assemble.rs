use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::{fs, process};

use assert_fs::prelude::*;
use pretty_assertions::assert_eq;

use git_wok::{DEFAULT_CONFIG_NAME, cmd, config::Config};

#[test]
fn assemble_creates_workspace_config() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let workspace_path = temp_dir.child("workspace");
    workspace_path.create_dir_all().unwrap();

    create_component(workspace_path.path(), "component-a");
    create_component(workspace_path.path(), "component-b");

    let config_path = workspace_path.path().join(DEFAULT_CONFIG_NAME);
    let mut output = Cursor::new(Vec::new());

    cmd::assemble(workspace_path.path(), &config_path, &mut output).unwrap();

    let config = Config::load(&config_path).unwrap();
    assert_eq!(config.repos.len(), 2);

    let mut repo_paths: Vec<_> = config
        .repos
        .iter()
        .map(|repo| repo.path.to_string_lossy().into_owned())
        .collect();
    repo_paths.sort();

    assert_eq!(
        repo_paths,
        vec![String::from("component-a"), String::from("component-b")]
    );
    assert!(config.repos.iter().all(|repo| repo.head == "main"));

    let gitmodules =
        fs::read_to_string(workspace_path.path().join(".gitmodules")).unwrap();
    assert!(gitmodules.contains("component-a"));
    assert!(gitmodules.contains("component-b"));

    assert!(is_submodule(workspace_path.path().join("component-a")));
    assert!(is_submodule(workspace_path.path().join("component-b")));
}

#[test]
fn assemble_is_idempotent() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let workspace_path = temp_dir.child("workspace");
    workspace_path.create_dir_all().unwrap();

    create_component(workspace_path.path(), "service-a");
    create_component(workspace_path.path(), "service-b");

    let config_path = workspace_path.path().join(DEFAULT_CONFIG_NAME);

    cmd::assemble(
        workspace_path.path(),
        &config_path,
        &mut Cursor::new(Vec::new()),
    )
    .unwrap();

    let first_config = Config::load(&config_path).unwrap();

    cmd::assemble(
        workspace_path.path(),
        &config_path,
        &mut Cursor::new(Vec::new()),
    )
    .unwrap();

    let second_config = Config::load(&config_path).unwrap();

    assert_eq!(first_config.dump().unwrap(), second_config.dump().unwrap());
}

fn create_component(base: &Path, name: &str) {
    let component_dir = base.join(name);
    fs::create_dir_all(&component_dir).unwrap();
}

fn is_submodule(path: PathBuf) -> bool {
    let git_path = path.join(".git");
    if !git_path.exists() {
        return false;
    }

    if git_path.is_file() {
        let content = fs::read_to_string(git_path).unwrap();
        content.contains("gitdir")
    } else {
        // fall back to git for existing repos
        let mut cmd = process::Command::new("git");
        cmd.arg("rev-parse")
            .arg("--is-inside-work-tree")
            .current_dir(path);
        let output = cmd.output().unwrap();
        output.status.success()
    }
}
