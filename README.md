# Git Wok

[![status-badge](https://ci.codeberg.org/api/badges/15540/status.svg)](https://ci.codeberg.org/repos/15540)

Git Wok manages multiple Git repositories as one workspace, with Git submodules
as the source of truth.

Use it to keep many component repos in sync for daily development, updates, and
releases from a single umbrella repository.

## Features

- Workspace setup for existing umbrellas (`init`) or directories of repos (`assemble`)
- Daily multirepo operations (`status`, `switch`, `lock`, `update`, `push`)
- Repo configuration management (`add`, `rm`)
- Release workflows across repos (`tag create`, `tag list`, `tag push`)
- Selective targeting: current-branch repos (default), `--all`, or explicit paths
- Per-repo `skip_for` controls in `wok.toml` for bulk commands
- Git LFS-aware `update` and `push` behavior
- Shell completion generation for Bash, Zsh, and Fish
- Authentication diagnostics with `test-auth`

## Agent Skills

AI agents working in a wok-managed workspace can install the [wok agent skills](https://codeberg.org/wok/skills) to operate multirepos correctly using the `wok` CLI:

```bash
npx skills add https://codeberg.org/wok/skills.git
```

Supports Cursor, Claude Code, Codex, Gemini CLI, OpenCode, and [65+ more agents](https://agentskills.io).

## Quick Start

```sh
# Install via Homebrew (macOS/Linux)
brew tap wok/wok https://codeberg.org/wok/homebrew-wok
brew trust wok/wok
brew install wok

# Or install from crates.io
cargo install git-wok

# In an existing umbrella repository with submodules
wok init

# Check workspace state
wok status

# Reconcile repos to the wok.toml state on the current branch
wok switch

# Reconcile repos using branch state from wok.toml on "main"
wok switch -b main

# Update subrepos and commit updated submodule pointers
wok update

# Push all repos (and umbrella)
wok push --all -u

# Create and push a signed release tag in all repos
wok tag --all create v1.0.0 --sign
wok tag --all push
```

## Documentation

- Main docs: <https://git-wok.dev/>
- Getting started: <https://git-wok.dev/getting-started/>
- CLI reference: <https://git-wok.dev/cli/>
- Wokfile reference: <https://git-wok.dev/wokfile/>

## Community

- Delta Chat:
  [Git Wok group](https://i.delta.chat/#667BD2FB6B122F4138F29A17861B4E257DCDFDB9&a=lig%40countzero.co&g=Git%20Wok&x=0FgEK_cMRZ6NMvG1PAekdJE3&i=9Jn9KZM9tErF-O0k8xvadsn_&s=DyV77Vq3p4y86HX9rRuOMvm2)
