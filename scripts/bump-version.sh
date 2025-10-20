#!/usr/bin/env bash
# Script to bump version in Cargo.toml and pyproject.toml
#
# This script updates the version number in both configuration files
# to ensure consistency across the project.

set -e  # Exit on error
set -u  # Exit on undefined variable
set -o pipefail  # Exit on pipe failure

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$REPO_ROOT/Cargo.toml"
PYPROJECT_TOML="$REPO_ROOT/pyproject.toml"

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗ Error:${NC} $1" >&2
    exit 1
}

print_warning() {
    echo -e "${YELLOW}⚠ Warning:${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_step() {
    echo -e "\n${BLUE}==>${NC} $1"
}

show_help() {
    cat << EOF
Usage: $(basename "$0") VERSION

Bump the version number in Cargo.toml and pyproject.toml.

ARGUMENTS:
    VERSION         The new version number (e.g., 1.0.0, 1.2.3-beta.1)

OPTIONS:
    -h, --help      Show this help message
    --dry-run       Show what would be changed without modifying files

EXAMPLES:
    $(basename "$0") 1.0.0
    $(basename "$0") 1.2.3-beta.1
    $(basename "$0") --dry-run 2.0.0

VERSION FORMAT:
    The version should follow semantic versioning (semver):
    - MAJOR.MINOR.PATCH (e.g., 1.0.0)
    - MAJOR.MINOR.PATCH-PRERELEASE (e.g., 1.0.0-beta.4)
    - MAJOR.MINOR.PATCH-PRERELEASE+BUILD (e.g., 1.0.0-beta.4+abc123)

EOF
}

validate_version() {
    local version="$1"

    # Check if version matches semver pattern
    # Allows: X.Y.Z, X.Y.Z-prerelease, X.Y.Z-prerelease+build
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.\-]+)?(\+[a-zA-Z0-9\.\-]+)?$ ]]; then
        print_error "Invalid version format: $version\nExpected format: X.Y.Z[-prerelease][+build]"
    fi
}

get_current_version() {
    local file="$1"
    local current_version=""

    if [[ "$file" == "$CARGO_TOML" ]]; then
        # Extract version from Cargo.toml
        current_version=$(grep '^version = ' "$file" | head -1 | sed 's/version = "\(.*\)"/\1/')
    elif [[ "$file" == "$PYPROJECT_TOML" ]]; then
        # Extract version from pyproject.toml
        current_version=$(grep '^version = ' "$file" | head -1 | sed 's/version = "\(.*\)"/\1/')
    fi

    echo "$current_version"
}

check_prerequisites() {
    print_step "Checking prerequisites"

    # Check if we're in the repository root
    if [[ ! -f "$CARGO_TOML" ]]; then
        print_error "Cargo.toml not found at: $CARGO_TOML"
    fi
    print_success "Found Cargo.toml"

    if [[ ! -f "$PYPROJECT_TOML" ]]; then
        print_error "pyproject.toml not found at: $PYPROJECT_TOML"
    fi
    print_success "Found pyproject.toml"

    # Check if sed is available
    if ! command -v sed &> /dev/null; then
        print_error "sed command not found. Please install sed."
    fi
}

show_current_versions() {
    print_step "Current versions"

    local cargo_version
    local pyproject_version

    cargo_version=$(get_current_version "$CARGO_TOML")
    pyproject_version=$(get_current_version "$PYPROJECT_TOML")

    print_info "Cargo.toml:     $cargo_version"
    print_info "pyproject.toml: $pyproject_version"

    if [[ "$cargo_version" != "$pyproject_version" ]]; then
        print_warning "Version mismatch detected between files"
    fi
}

bump_version() {
    local new_version="$1"
    local dry_run="$2"

    print_step "Bumping version to $new_version"

    if [[ "$dry_run" == "true" ]]; then
        print_warning "DRY RUN: No files will be modified"
    fi

    # Update Cargo.toml
    if [[ "$dry_run" == "true" ]]; then
        print_info "Would update Cargo.toml: version = \"$new_version\""
    else
        # Use sed to replace version in Cargo.toml (first occurrence after [package])
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS requires empty string after -i
            sed -i '' "0,/^version = /s/^version = .*/version = \"$new_version\"/" "$CARGO_TOML"
        else
            # Linux
            sed -i "0,/^version = /s/^version = .*/version = \"$new_version\"/" "$CARGO_TOML"
        fi
        print_success "Updated Cargo.toml"
    fi

    # Update pyproject.toml
    if [[ "$dry_run" == "true" ]]; then
        print_info "Would update pyproject.toml: version = \"$new_version\""
    else
        # Use sed to replace version in pyproject.toml (first occurrence)
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS requires empty string after -i
            sed -i '' "0,/^version = /s/^version = .*/version = \"$new_version\"/" "$PYPROJECT_TOML"
        else
            # Linux
            sed -i "0,/^version = /s/^version = .*/version = \"$new_version\"/" "$PYPROJECT_TOML"
        fi
        print_success "Updated pyproject.toml"
    fi
}

verify_changes() {
    print_step "Verifying changes"

    local cargo_version
    local pyproject_version

    cargo_version=$(get_current_version "$CARGO_TOML")
    pyproject_version=$(get_current_version "$PYPROJECT_TOML")

    print_info "Cargo.toml:     $cargo_version"
    print_info "pyproject.toml: $pyproject_version"

    if [[ "$cargo_version" == "$pyproject_version" ]]; then
        print_success "Versions are in sync"
    else
        print_error "Version mismatch after update!"
    fi
}

main() {
    local new_version=""
    local dry_run="false"

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            --dry-run)
                dry_run="true"
                shift
                ;;
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

    # Check if version was provided
    if [[ -z "$new_version" ]]; then
        print_error "Version number required.\nUsage: $(basename "$0") VERSION\nUse --help for more information."
    fi

    # Validate version format
    validate_version "$new_version"

    # Run the workflow
    check_prerequisites
    show_current_versions
    bump_version "$new_version" "$dry_run"

    if [[ "$dry_run" == "false" ]]; then
        verify_changes

        # Success summary
        echo
        print_success "Version bump complete!"
        print_info "Version updated to: $new_version"
        echo
        print_info "Next steps:"
        echo "  1. Review the changes: git diff"
        echo "  2. Run tests: cargo test --all"
        echo "  3. Commit the changes: git add Cargo.toml pyproject.toml"
        echo "  4. Create a commit: git commit -m \"Bump version to $new_version\""
        echo "  5. Create a tag: git tag v$new_version"
    else
        echo
        print_info "Dry run complete. No files were modified."
    fi
}

# Run main function
main "$@"
