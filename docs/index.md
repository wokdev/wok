# Git Wok

A powerful multirepo management tool built with Rust.

---

Git Wok manages multiple git repositories as part of a single workspace, using git submodules under the hood. Think of it as a package manager for your multi-repository projects—similar to how `cargo`, `poetry`, or `npm` manage dependencies.

## Key Features

- **9 Complete Commands** for comprehensive multi-repo management
- **Package Management**: Initialize workspaces, add/remove repos, update dependencies, lock state
- **Development Flow**: Branch switching, pushing, status checking across all repos
- **Release Management**: Create, sign, and push tags across multiple repositories
- **Smart Controls**: Selective repo targeting, per-repo skip lists, branch creation options
- **TOML Configuration**: Simple, version-controlled `wok.toml` file

[Get started](getting-started.md){ .md-button .md-button--primary }
[View Commands](cli.md){ .md-button }
