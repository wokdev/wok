set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Convenience commands for this repo.
#
# Run `just` or `just help` to see available recipes.

default:
    @just --list

help:
    @just --list

# --- scripts/ wrappers --------------------------------------------------------

# Bump version in pyproject.toml and refresh uv.lock.
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

# Update Homebrew formula (homebrew-wok submodule) to VERSION.
update-formula VERSION:
    bash scripts/update-formula.sh "{{VERSION}}"

# Show what would change, without modifying the formula.
update-formula-dry VERSION:
    bash scripts/update-formula.sh --dry-run "{{VERSION}}"

# Release flow. VERSION is a semver like 1.2.3 or 1.2.3-rc.1.
# uv publish uploads to PyPI. Run it only when a release is intended.
release VERSION:
    just bump-version "{{VERSION}}"
    uv build
    just update-site
    git -C site add -A
    git -C site commit --allow-empty -m "Update docs for v{{VERSION}}"
    git add pyproject.toml uv.lock site
    git commit --allow-empty -m "Bump version to v{{VERSION}}"
    wok push
    uv publish
    just update-formula "{{VERSION}}"
    git -C homebrew-wok add -A
    git -C homebrew-wok commit --allow-empty -m "Update wok formula to v{{VERSION}}"
    git add homebrew-wok
    git commit --allow-empty -m "Update Homebrew formula to v{{VERSION}}"
    wok push
    wok tag create "v{{VERSION}}" -sm "v{{VERSION}}"
    wok tag push

# --- common dev commands ------------------------------------------------------

build:
    uv build

test:
    uv run pytest

fmt:
    uv run ruff format .
    uv run ruff check --fix .

lint:
    uv run ruff format --check .
    uv run ruff check .
    uv run pyrefly check

docs-serve:
    uv run mkdocs serve
