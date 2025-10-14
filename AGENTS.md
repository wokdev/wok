# Repository Guidelines

## Project Structure & Module Organization
Wok is a Rust CLI. Core workspace logic lives in `src/lib.rs` and supporting modules under `src/cmd` for each command (`init`, `tag`, `push`, etc.). CLI entry binaries sit in `src/bin`, while `src/config.rs` and `src/repo.rs` hold configuration and repository helpers. Integration tests live in `tests/cmd`, with fixture workspaces in `tests/data`. User-facing docs and MkDocs sources are under `docs/`; the generated static site is in `site/`.

## Build, Test, and Development Commands
- `cargo build --all` compiles the CLI and validates dependencies.
- `cargo run -- <args>` executes the CLI locally; for example `cargo run -- status`.
- `cargo test --all` runs unit and integration suites; tack on `-- --nocapture` to keep CLI output visible.
- `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings` enforce formatting and linting; both must pass before PR review.
- `mkdocs serve` (Python 3.11+) previews the documentation site at http://localhost:8000.

## Coding Style & Naming Conventions
Rust files follow the project `rustfmt.toml` (88-column max, wrapped comments, crate-level import grouping). Use four-space indentation, snake_case for modules/functions, CamelCase for types, and SCREAMING_SNAKE_CASE for constants. Error messages should be concise and actionable. Keep modules cohesive: each command gets its own file in `src/cmd`, and shared helpers belong in `src/lib.rs` or a dedicated module.

## Testing Guidelines
Prefer integration scenarios mirroring CLI usage; new commands should ship with a `tests/cmd/<command>.rs` case exercising success and failure flows. Use descriptive test functions like `fn init_creates_workspace()` and reuse fixtures from `tests/data`. Run `cargo test --all` before opening a PR; if a change impacts docs examples, add a smoke test to ensure serialized output stays stable. Aim to retain coverage for each CLI command path.

## Commit & Pull Request Guidelines
Write commit subjects in the imperative mood (“Add tag force flag”), keep them under ~72 characters, and group related changes together. Reference issues with `Refs #123` or `Fixes #123` in the body when relevant. Pull requests should include: a concise summary, a checklist of commands run (`cargo fmt`, `cargo test`, `cargo clippy`), and screenshots or logs for user-facing changes. Request review after CI passes and link any follow-up tasks in the description.
