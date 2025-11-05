use git_wok::config::Config;
use rstest::*;
use std::fs;

use super::*;

#[rstest(repo_sample())]
fn migrates_experimental_version_to_stable(repo_sample: TestRepo) {
    let config_path = repo_sample.config_path();

    // Write a config with experimental version
    let experimental_config = r#"version = "1.0-experimental"
repo = []
"#;
    fs::write(&config_path, experimental_config).unwrap();

    // Load the config (should trigger migration)
    let config = Config::load(&config_path).unwrap();

    // Verify version was migrated in memory
    assert_eq!(config.version, "1.0");

    // Verify version was migrated on disk
    let file_content = fs::read_to_string(&config_path).unwrap();
    assert!(file_content.contains(r#"version = "1.0""#));
    assert!(!file_content.contains("experimental"));
}

#[rstest(repo_sample())]
fn stable_version_loads_without_modification(repo_sample: TestRepo) {
    let config_path = repo_sample.config_path();

    // Write a config with stable version
    let stable_config = r#"version = "1.0"
repo = []
"#;
    fs::write(&config_path, stable_config).unwrap();
    let original_mtime = fs::metadata(&config_path).unwrap().modified().unwrap();

    // Small delay to ensure mtime would change if file was modified
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Load the config
    let config = Config::load(&config_path).unwrap();

    // Verify version is correct
    assert_eq!(config.version, "1.0");

    // Verify file was not modified (mtime unchanged)
    let new_mtime = fs::metadata(&config_path).unwrap().modified().unwrap();
    assert_eq!(original_mtime, new_mtime);
}

#[rstest(repo_sample())]
fn migrates_experimental_version_with_repos(repo_sample: TestRepo) {
    let config_path = repo_sample.config_path();

    // Write a config with experimental version and repos
    let experimental_config = r#"version = "1.0-experimental"

[[repo]]
path = "sub-a"
head = "main"

[[repo]]
path = "sub-b"
head = "develop"
skip_for = ["push", "tag"]
"#;
    fs::write(&config_path, experimental_config).unwrap();

    // Load the config (should trigger migration)
    let config = Config::load(&config_path).unwrap();

    // Verify version was migrated in memory
    assert_eq!(config.version, "1.0");

    // Verify repos are preserved
    assert_eq!(config.repos.len(), 2);
    assert_eq!(config.repos[0].path.to_str().unwrap(), "sub-a");
    assert_eq!(config.repos[0].head, "main");
    assert_eq!(config.repos[1].path.to_str().unwrap(), "sub-b");
    assert_eq!(config.repos[1].head, "develop");
    assert_eq!(config.repos[1].skip_for, vec!["push", "tag"]);

    // Verify version was migrated on disk
    let file_content = fs::read_to_string(&config_path).unwrap();
    assert!(file_content.contains(r#"version = "1.0""#));
    assert!(!file_content.contains("experimental"));
    
    // Verify repos are still in the file
    assert!(file_content.contains("sub-a"));
    assert!(file_content.contains("sub-b"));
    assert!(file_content.contains("develop"));
}
