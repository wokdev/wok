fn main() {
    eprintln!("╔══════════════════════════════════════════════════════════════╗");
    eprintln!("║                                                              ║");
    eprintln!("║  The 'wok-dev' package has been renamed to 'git-wok'        ║");
    eprintln!("║                                                              ║");
    eprintln!("║  Please uninstall this package and install git-wok instead: ║");
    eprintln!("║                                                              ║");
    eprintln!("║    cargo uninstall wok-dev                                   ║");
    eprintln!("║    cargo install git-wok                                     ║");
    eprintln!("║                                                              ║");
    eprintln!("║  For more information, visit:                                ║");
    eprintln!("║    https://git-wok.dev/                                      ║");
    eprintln!("║                                                              ║");
    eprintln!("╚══════════════════════════════════════════════════════════════╝");
    std::process::exit(1);
}
