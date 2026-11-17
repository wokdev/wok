#!/usr/bin/env bash
# Point homebrew-wok/Formula/wok.rb at a PyPI sdist for VERSION.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FORMULA="$REPO_ROOT/homebrew-wok/Formula/wok.rb"

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

Update the Homebrew formula url and sha256 for a PyPI sdist.

Uses dist/wok-VERSION.tar.gz when that file exists. Otherwise downloads
https://files.pythonhosted.org/packages/source/w/wok/wok-VERSION.tar.gz.
EOF
}

validate_version() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.\-]+)?(\+[a-zA-Z0-9\.\-]+)?$ ]]; then
        print_error "Invalid version format: $version\nExpected format: X.Y.Z[-prerelease][+build]"
    fi
}

compute_sha256() {
    local file="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        print_error "Neither sha256sum nor shasum found.\nInstall coreutils: sudo dnf install coreutils"
    fi
}

main() {
    local version=""
    local dry_run="false"

    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help) show_help; exit 0 ;;
            --dry-run) dry_run="true"; shift ;;
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
        print_error "Version number required.\nUsage: $(basename "$0") VERSION"
    fi
    validate_version "$version"

    if [[ ! -f "$FORMULA" ]]; then
        print_error "Formula not found at: $FORMULA"
    fi

    local archive="$REPO_ROOT/dist/wok-${version}.tar.gz"
    local url="https://files.pythonhosted.org/packages/source/w/wok/wok-${version}.tar.gz"
    local tmp=""
    if [[ ! -f "$archive" ]]; then
        if ! command -v curl >/dev/null 2>&1; then
            print_error "curl not found. Install it: sudo dnf install curl"
        fi
        tmp="$(mktemp --suffix=.tar.gz)"
        print_info "Downloading $url"
        if ! curl -fsSL "$url" -o "$tmp"; then
            rm -f "$tmp"
            print_error "Failed to download $url. Build it first with: uv build"
        fi
        archive="$tmp"
    else
        print_info "Using local sdist: $archive"
    fi

    local sha
    sha="$(compute_sha256 "$archive")"
    if [[ -n "$tmp" ]]; then
        rm -f "$tmp"
    fi
    print_success "sha256: $sha"

    if [[ "$dry_run" == "true" ]]; then
        print_warning "DRY RUN: No files will be modified"
        print_info "Would set url to:    $url"
        print_info "Would set sha256 to: $sha"
        exit 0
    fi

    python3 - "$FORMULA" "$url" "$sha" << 'PY'
import pathlib, sys
path, url, sha = sys.argv[1:]
file = pathlib.Path(path)
text = file.read_text()
import re
text2, n1 = re.subn(r'url "[^"]+"', f'url "{url}"', text, count=1)
text3, n2 = re.subn(r'sha256 "[^"]+"', f'sha256 "{sha}"', text2, count=1)
if n1 != 1 or n2 != 1:
    raise SystemExit("Could not find url or sha256 in the formula")
file.write_text(text3)
PY
    print_success "Updated formula to v${version}"
}

main "$@"
