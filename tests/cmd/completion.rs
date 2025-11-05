use std::env;
use std::process::Command;

#[test]
fn completion_bash_generates_script() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("completion")
        .arg("bash")
        .output()
        .expect("Failed to execute wok completion bash");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_wok()"));
    assert!(stdout.contains("complete -F _wok"));
}

#[test]
fn completion_fish_generates_script() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("completion")
        .arg("fish")
        .output()
        .expect("Failed to execute wok completion fish");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("__fish_wok"));
}

#[test]
fn completion_zsh_generates_script() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("completion")
        .arg("zsh")
        .output()
        .expect("Failed to execute wok completion zsh");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#compdef wok"));
}

#[test]
fn completion_defaults_to_bash() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("completion")
        .output()
        .expect("Failed to execute wok completion");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_wok()"));
    assert!(stdout.contains("complete -F _wok"));
}

#[test]
fn completion_help_shows_available_shells() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("completion")
        .arg("--help")
        .output()
        .expect("Failed to execute wok completion --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bash"));
    assert!(stdout.contains("fish"));
    assert!(stdout.contains("zsh"));
    assert!(stdout.contains("[default: bash]"));
}
