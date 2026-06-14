#!/usr/bin/env bash
# Script to update the Homebrew formula (homebrew-wok/Formula/wok.rb) for a new release.
#
# Rewrites the url and sha256 lines in the formula to match the crates.io tarball
# for the given VERSION. Prefers a locally packaged crate (target/package/) when
# available so the hash matches the upload exactly; otherwise downloads from crates.io.

set -e
set -u
set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FORMULA="$REPO_ROOT/homebrew-wok/Formula/wok.rb"

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
Usage: $(basename "$0") [--dry-run] VERSION

Update the Homebrew formula (homebrew-wok/Formula/wok.rb) to a new VERSION.

ARGUMENTS:
    VERSION         The new version number (e.g., 1.5.0)

OPTIONS:
    -h, --help      Show this help message
    --dry-run       Show what would be changed without modifying the formula

EXAMPLES:
    $(basename "$0") 1.5.0
    $(basename "$0") --dry-run 1.6.0

VERSION FORMAT:
    The version should follow semantic versioning (semver):
    - MAJOR.MINOR.PATCH (e.g., 1.5.0)
    - MAJOR.MINOR.PATCH-PRERELEASE (e.g., 1.6.0-rc.1)

EOF
}

validate_version() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.\-]+)?(\+[a-zA-Z0-9\.\-]+)?$ ]]; then
        print_error "Invalid version format: $version\nExpected format: X.Y.Z[-prerelease][+build]"
    fi
}

check_sha256_tool() {
    if command -v sha256sum &> /dev/null; then
        echo "sha256sum"
    elif command -v shasum &> /dev/null; then
        echo "shasum"
    else
        print_error "Neither sha256sum nor shasum found.\nInstall coreutils: sudo dnf install coreutils"
    fi
}

compute_sha256() {
    local file="$1"
    local tool
    tool="$(check_sha256_tool)"

    if [[ "$tool" == "sha256sum" ]]; then
        sha256sum "$file" | awk '{print $1}'
    else
        shasum -a 256 "$file" | awk '{print $1}'
    fi
}

check_prerequisites() {
    print_step "Checking prerequisites"

    if [[ ! -f "$FORMULA" ]]; then
        print_error "Formula not found at: $FORMULA\nEnsure the homebrew-wok submodule is initialized:\n  git submodule update --init homebrew-wok"
    fi
    print_success "Found formula: $FORMULA"

    check_sha256_tool > /dev/null
    print_success "sha256 tool available"
}

get_sha256() {
    local version="$1"
    local crate_name="git-wok-${version}.crate"
    local local_crate="$REPO_ROOT/target/package/${crate_name}"
    local crate_url="https://static.crates.io/crates/git-wok/${crate_name}"

    if [[ -f "$local_crate" ]]; then
        print_info "Using locally packaged crate: $local_crate"
        compute_sha256 "$local_crate"
    else
        print_info "Downloading crate from crates.io..."
        if ! command -v curl &> /dev/null; then
            print_error "curl not found. Install it: sudo dnf install curl"
        fi
        local tmp_file
        tmp_file="$(mktemp --suffix=.crate)"
        if ! curl -fsSL "$crate_url" -o "$tmp_file"; then
            rm -f "$tmp_file"
            print_error "Failed to download: $crate_url"
        fi
        local hash
        hash="$(compute_sha256 "$tmp_file")"
        rm -f "$tmp_file"
        echo "$hash"
    fi
}

update_formula() {
    local version="$1"
    local sha256="$2"
    local dry_run="$3"
    local new_url="https://static.crates.io/crates/git-wok/git-wok-${version}.crate"

    print_step "Updating formula to v${version}"

    if [[ "$dry_run" == "true" ]]; then
        print_warning "DRY RUN: No files will be modified"
        print_info "Would set url to:    $new_url"
        print_info "Would set sha256 to: $sha256"
        return
    fi

    # Rewrite url line
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s|url \"https://static\.crates\.io/crates/git-wok/git-wok-[^\"]*\"|url \"${new_url}\"|" "$FORMULA"
        sed -i '' "s|sha256 \"[^\"]*\"|sha256 \"${sha256}\"|" "$FORMULA"
    else
        sed -i "s|url \"https://static\.crates\.io/crates/git-wok/git-wok-[^\"]*\"|url \"${new_url}\"|" "$FORMULA"
        sed -i "s|sha256 \"[^\"]*\"|sha256 \"${sha256}\"|" "$FORMULA"
    fi

    print_success "Updated url to:    $new_url"
    print_success "Updated sha256 to: $sha256"
}

main() {
    local version=""
    local dry_run="false"

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
                if [[ -z "$version" ]]; then
                    version="$1"
                else
                    print_error "Too many arguments. Expected one version number."
                fi
                shift
                ;;
        esac
    done

    if [[ -z "$version" ]]; then
        print_error "Version number required.\nUsage: $(basename "$0") VERSION\nUse --help for more information."
    fi

    validate_version "$version"
    check_prerequisites

    print_step "Computing sha256 for v${version}"
    local sha256
    sha256="$(get_sha256 "$version")"
    print_success "sha256: $sha256"

    update_formula "$version" "$sha256" "$dry_run"

    if [[ "$dry_run" == "false" ]]; then
        echo
        print_success "Formula update complete!"
        print_info "Version updated to: $version"
        echo
        print_info "Next steps:"
        echo "  1. Commit and push the formula: git -C homebrew-wok add -A && git -C homebrew-wok commit -m \"Update wok formula to v${version}\""
        echo "  2. Update the umbrella pointer: git add homebrew-wok && git commit -m \"Update Homebrew formula to v${version}\""
        echo "  3. Push all repos: wok push"
    else
        echo
        print_info "Dry run complete. No files were modified."
    fi
}

main "$@"
