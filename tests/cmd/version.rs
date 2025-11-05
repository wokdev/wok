use std::env;
use std::process::Command;

#[test]
fn version_flag_works() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("--version")
        .output()
        .expect("Failed to execute wok --version");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wok"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn version_short_flag_works() {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wok_binary = format!("{}/target/debug/wok", cargo_manifest_dir);

    let output = Command::new(&wok_binary)
        .arg("-V")
        .output()
        .expect("Failed to execute wok -V");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wok"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}
