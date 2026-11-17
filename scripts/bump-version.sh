#!/usr/bin/env bash
# Bump the version in pyproject.toml and refresh uv.lock.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PYPROJECT_TOML="$REPO_ROOT/pyproject.toml"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_error() { echo -e "${RED}✗ Error:${NC} $1" >&2; exit 1; }
print_warning() { echo -e "${YELLOW}⚠ Warning:${NC} $1"; }
print_info() { echo -e "${BLUE}ℹ${NC} $1"; }
print_step() { echo -e "\n${BLUE}==>${NC} $1"; }

show_help() {
    cat << EOF
Usage: $(basename "$0") [--dry-run] VERSION

Bump the version in pyproject.toml and refresh uv.lock.

ARGUMENTS:
    VERSION         The new version number (e.g., 1.0.0, 1.2.3-beta.1)

OPTIONS:
    -h, --help      Show this help message
    --dry-run       Show what would be changed without modifying files
EOF
}

validate_version() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.\-]+)?(\+[a-zA-Z0-9\.\-]+)?$ ]]; then
        print_error "Invalid version format: $version\nExpected format: X.Y.Z[-prerelease][+build]"
    fi
}

current_version() {
    grep '^version = ' "$PYPROJECT_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

main() {
    local new_version=""
    local dry_run="false"

    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help) show_help; exit 0 ;;
            --dry-run) dry_run="true"; shift ;;
            *)
                if [[ -z "$new_version" ]]; then
                    new_version="$1"
                else
                    print_error "Too many arguments. Expected one version number."
                fi
                shift
                ;;
        esac
    done

    if [[ -z "$new_version" ]]; then
        print_error "Version number required.\nUsage: $(basename "$0") VERSION"
    fi
    validate_version "$new_version"

    if [[ ! -f "$PYPROJECT_TOML" ]]; then
        print_error "pyproject.toml not found at: $PYPROJECT_TOML"
    fi

    print_step "Current version"
    print_info "pyproject.toml: $(current_version)"

    print_step "Bumping version to $new_version"
    if [[ "$dry_run" == "true" ]]; then
        print_warning "DRY RUN: No files will be modified"
        print_info "Would set pyproject.toml version to \"$new_version\" and run uv lock"
        exit 0
    fi

    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "0,/^version = /s/^version = .*/version = \"$new_version\"/" "$PYPROJECT_TOML"
    else
        sed -i "0,/^version = /s/^version = .*/version = \"$new_version\"/" "$PYPROJECT_TOML"
    fi
    print_success "Updated pyproject.toml"

    if ! command -v uv >/dev/null 2>&1; then
        print_error "uv not found. Install it: curl -LsSf https://astral.sh/uv/install.sh | sh"
    fi
    (cd "$REPO_ROOT" && uv lock)
    print_success "Refreshed uv.lock"

    if [[ "$(current_version)" != "$new_version" ]]; then
        print_error "pyproject.toml version is $(current_version), expected $new_version"
    fi
    print_success "Version bump complete: $new_version"
}

main "$@"
