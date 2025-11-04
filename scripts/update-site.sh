#!/usr/bin/env bash
# Script to build docs with MkDocs and update the site directory
#
# This script automates the process of:
# 1. Building documentation with MkDocs from the docs/ directory
# 2. Generating static site files into the site/ directory

set -e  # Exit on error
set -u  # Exit on undefined variable
set -o pipefail  # Exit on pipe failure

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SITE_DIR="$REPO_ROOT/site"
DOCS_DIR="$REPO_ROOT/docs"

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
VERBOSE=false
CLEAN=true

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

log_verbose() {
    if [[ "$VERBOSE" == true ]]; then
        echo -e "${BLUE}[VERBOSE]${NC} $1"
    fi
}

show_help() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Build documentation with MkDocs and update the site directory.

OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Show verbose output
    --no-clean          Don't clean the site directory before building

EXAMPLES:
    $(basename "$0")                # Build docs with clean build
    $(basename "$0") --verbose      # Show detailed output
    $(basename "$0") --no-clean     # Build without cleaning first

EOF
}

check_prerequisites() {
    print_step "Checking prerequisites" >&2

    # Check if we're in the repository root
    if [[ ! -f "$REPO_ROOT/Cargo.toml" ]]; then
        print_error "Not in repository root. Please run this script from the wok repository."
    fi
    log_verbose "Repository root: $REPO_ROOT" >&2
    print_success "Running from repository root" >&2

    # Check if docs directory exists
    if [[ ! -d "$DOCS_DIR" ]]; then
        print_error "Documentation directory not found at: $DOCS_DIR"
    fi
    log_verbose "Docs directory: $DOCS_DIR" >&2
    print_success "Documentation directory found" >&2

    # Check for mkdocs (try uv first, then system)
    local mkdocs_cmd=""
    if command -v uv &> /dev/null; then
        if uv run mkdocs --version &> /dev/null 2>&1; then
            mkdocs_cmd="uv run mkdocs"
            log_verbose "Using mkdocs via uv" >&2
        fi
    fi

    if [[ -z "$mkdocs_cmd" ]] && command -v mkdocs &> /dev/null; then
        mkdocs_cmd="mkdocs"
        log_verbose "Using system mkdocs" >&2
    fi

    if [[ -z "$mkdocs_cmd" ]]; then
        print_error "MkDocs not found. Please install it:\n  uv sync\n  or\n  pip install mkdocs"
    fi

    echo "$mkdocs_cmd"
}


build_docs() {
    local mkdocs_cmd="$1"

    print_step "Building documentation with MkDocs"

    cd "$REPO_ROOT"

    local build_args="build"
    if [[ "$CLEAN" == true ]]; then
        build_args="$build_args --clean"
        log_verbose "Running: $mkdocs_cmd build --clean"
    else
        log_verbose "Running: $mkdocs_cmd build"
    fi

    if [[ "$VERBOSE" == true ]]; then
        $mkdocs_cmd $build_args
    else
        $mkdocs_cmd $build_args --quiet
    fi

    print_success "Documentation built successfully"
    print_info "Output directory: $SITE_DIR"
}


main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            --no-clean)
                CLEAN=false
                shift
                ;;
            *)
                print_error "Unknown option: $1\nUse --help for usage information."
                ;;
        esac
    done

    log_verbose "Verbose: $VERBOSE"
    log_verbose "Clean build: $CLEAN"

    # Run the workflow
    local mkdocs_cmd
    mkdocs_cmd=$(check_prerequisites)
    build_docs "$mkdocs_cmd"

    # Success summary
    echo
    print_success "Site build complete!"
    print_info "The documentation site has been generated in: $SITE_DIR"
}

# Run main function
main "$@"
