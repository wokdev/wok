fn main() -> std::process::ExitCode {
    eprintln!(
        "\
git-wok on crates.io is discontinued. wok is now the Python package `wok`.

Install it with one of:
  uv tool install wok
  pipx install wok
  brew install wok/wok/wok

Then run `cargo uninstall git-wok`."
    );
    std::process::ExitCode::from(1)
}
