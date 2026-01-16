set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Convenience commands for this repo.
#
# Run `just` or `just help` to see available recipes.

default:
    @just --list

help:
    @just --list

# --- scripts/ wrappers --------------------------------------------------------

# Bump version in Cargo.toml and pyproject.toml.
bump-version VERSION:
    bash scripts/bump-version.sh "{{VERSION}}"

# Show what would change, without modifying files.
bump-version-dry VERSION:
    bash scripts/bump-version.sh --dry-run "{{VERSION}}"

# Build docs and regenerate site/ (generated output; commit if desired).
update-site:
    bash scripts/update-site.sh

update-site-verbose:
    bash scripts/update-site.sh --verbose

update-site-no-clean:
    bash scripts/update-site.sh --no-clean

update-site-verbose-no-clean:
    bash scripts/update-site.sh --verbose --no-clean

# Release flow. VERSION is a semver like 1.2.3 or 1.2.3-rc.1.
release VERSION:
    just bump-version "{{VERSION}}"
    just update-site
    git -C site add -A
    git -C site commit -m "Update docs for v{{VERSION}}"
    git add Cargo.toml Cargo.lock pyproject.toml uv.lock site
    git commit -m "Bump version to v{{VERSION}}"
    wok push
    cargo publish
    wok tag create "v{{VERSION}}" -sm "v{{VERSION}}"
    wok tag push

# --- common dev commands ------------------------------------------------------

build:
    cargo build --all

test:
    cargo test --all

fmt:
    cargo fmt --all

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

docs-serve:
    if command -v uv >/dev/null 2>&1; then \
      uv run mkdocs serve; \
    else \
      mkdocs serve; \
    fi
