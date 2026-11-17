# Git Wok

Git Wok manages several Git repositories as one workspace. The umbrella
repository stores `wok.toml`, and committed submodule pointers are the lock.

The `wok` command is a Python package. It runs `git` and `git lfs`, so your
hooks, credentials, and Git configuration apply to every repository it touches.

## Install

```sh
uv tool install wok
```

```sh
pipx install wok
```

```sh
brew tap wok/wok https://codeberg.org/wok/homebrew-wok
brew install wok
```

Git is required. Install `git-lfs` as well when a repository uses Git LFS.
On Fedora:

```sh
sudo dnf install git git-lfs
git lfs install
```

The old `git-wok` crate on crates.io is discontinued. If it is still installed:

```sh
cargo uninstall git-wok
```

## Use

```sh
wok init
wok status
wok switch --all
wok lock
wok update
wok push
```

`wok --help` lists every command. Global `-j` / `--jobs` controls how many
repositories run at once (default `min(8, targets)`, also `WOK_JOBS`). On a
terminal, parallel runs show a live section per repository. `-j1` keeps Git
attached to the terminal so prompts and hooks that need a TTY can run.

## Develop

```sh
uv sync
uv run wok status
uv run pytest
uv run ruff check
uv run ruff format --check
uv run pyrefly check
```

Documentation sources live in `docs/`. Preview them with `uv run mkdocs serve`.
The generated `site/` directory is not edited by hand.
