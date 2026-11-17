# Repository Guidelines

## Project Structure & Module Organization
Wok is a Python CLI. The package lives in `src/wok`: `cli.py` is the Typer entry point, `config.py` reads and writes `wok.toml`, `repo.py` and `git.py` call the `git` command, and `src/wok/cmd` holds one module per command. Integration tests live in `tests/cmd`, with `wok.toml` fixtures in `tests/data`. User-facing docs and MkDocs sources are under `docs/`. The generated static site is in `site/`.

## Generated content (do not edit)
- The `site/` directory is generated and is controlled only by the user. Do not modify anything under `site/` in agent changes.

## Build, Test, and Development Commands
- `uv sync` installs the package and the dev tools.
- `uv run wok <args>` executes the CLI locally; for example `uv run wok status`.
- `uv run pytest` runs the integration suite.
- `uv run ruff format --check` and `uv run ruff check` enforce formatting and the configured lint rules.
- `uv run pyrefly check` type-checks `src` and `tests`. It must report no errors.
- `uv run mkdocs serve` previews the documentation site at http://localhost:8000.

## Coding Style & Naming Conventions
Python files use Ruff (88-column max). Use four-space indentation, snake_case for modules and functions, CamelCase for types, and SCREAMING_SNAKE_CASE for constants. Error messages should be concise and actionable. Keep modules cohesive: each command gets its own file in `src/wok/cmd`.

## Testing Guidelines
Prefer integration scenarios mirroring CLI usage. New commands should ship with a `tests/cmd/test_<command>.py` case exercising success and failure flows. Reuse fixtures from `tests/data`. Run `uv run pytest` before opening a PR.

## Commit & Pull Request Guidelines
Write commit subjects in the imperative mood, keep them under ~72 characters, and group related changes together. Use conventional prefixes (`feat`, `fix`, `chore`, `docs`, `test`, `refactor`). Reference issues with `Refs #123` or `Fixes #123` in the body when relevant. Pull requests should include a concise summary and the commands you ran (`uv run ruff format --check`, `uv run ruff check`, `uv run pyrefly check`, `uv run pytest`).
