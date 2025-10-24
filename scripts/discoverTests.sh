#!/bin/bash
#
# discoverTests.sh - Discover all test suites in the project
#
# This script outputs test suite identifiers, one per line, that can be used
# with `cargo test`. It discovers:
# 1. Library test modules (e.g., core::mana, game::actions)
# 2. Integration tests (e.g., card_loading, database_load)
# 3. Binary test modules (if any)
#
# Usage:
#   ./scripts/discoverTests.sh [OPTIONS]
#
# Options:
#   --lib           Only show library test modules
#   --integration   Only show integration tests
#   --bins          Only show binary test modules
#   --all           Show all test suites (default)
#   --help          Show this help message
#
# Output format:
#   One test suite identifier per line, suitable for use with:
#   cargo test <identifier>
#
# Examples:
#   # List all test suites
#   ./scripts/discoverTests.sh
#
#   # List only library test modules
#   ./scripts/discoverTests.sh --lib
#
#   # Run all lib tests in parallel using xargs
#   ./scripts/discoverTests.sh --lib | xargs -I {} -P 4 cargo test --lib {}
#
#   # Time each test suite with hyperfine
#   for suite in $(./scripts/discoverTests.sh --lib); do
#     hyperfine --warmup 1 "cargo test --lib $suite"
#   done

set -euo pipefail

# Default: show all
SHOW_LIB=false
SHOW_INTEGRATION=false
SHOW_BINS=false
SHOW_ALL=true

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --lib)
            SHOW_LIB=true
            SHOW_ALL=false
            shift
            ;;
        --integration)
            SHOW_INTEGRATION=true
            SHOW_ALL=false
            shift
            ;;
        --bins)
            SHOW_BINS=true
            SHOW_ALL=false
            shift
            ;;
        --all)
            SHOW_ALL=true
            shift
            ;;
        --help|-h)
            sed -n '/^# discoverTests.sh/,/^$/p' "$0" | grep "^#" | grep -v "^#!/" | sed 's/^# //' | sed 's/^#//'
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
done

# If specific options selected, override SHOW_ALL
if [[ "$SHOW_LIB" == "true" ]] || [[ "$SHOW_INTEGRATION" == "true" ]] || [[ "$SHOW_BINS" == "true" ]]; then
    SHOW_ALL=false
fi

# If no specific options, show all
if [[ "$SHOW_ALL" == "true" ]]; then
    SHOW_LIB=true
    SHOW_INTEGRATION=true
    SHOW_BINS=true
fi

# Function to discover library test modules
discover_lib_modules() {
    # Get list of all tests, extract module paths
    cargo test --lib -- --list 2>/dev/null | \
        grep '::tests::' | \
        sed 's/: test$//' | \
        sed -E 's/^([^:]+::[^:]+)::tests::.*/\1/' | \
        sort -u | \
        # Also handle top-level modules like undo::tests, zones::tests
        sed -E 's/^([^:]+)::tests::.*/\1/' | \
        sort -u
}

# Function to discover integration tests
discover_integration_tests() {
    # Use cargo metadata to find test targets
    cargo metadata --format-version 1 --no-deps 2>/dev/null | \
        jq -r '.packages[0].targets[] | select(.kind[] == "test") | .name'
}

# Function to discover binary test modules
discover_bin_modules() {
    # Get list of all binaries
    local bins=$(cargo metadata --format-version 1 --no-deps 2>/dev/null | \
        jq -r '.packages[0].targets[] | select(.kind[] == "bin") | .name')

    # For each binary, check if it has tests
    for bin in $bins; do
        local test_count=$(cargo test --bin "$bin" -- --list 2>/dev/null | grep -c '::tests::' || true)
        if [[ $test_count -gt 0 ]]; then
            # Output the test modules for this binary
            cargo test --bin "$bin" -- --list 2>/dev/null | \
                grep '::tests::' | \
                sed 's/: test$//' | \
                sed -E 's/^([^:]+::[^:]+)::tests::.*/\1/' | \
                sort -u
        fi
    done
}

# Output test suites based on flags
if [[ "$SHOW_LIB" == "true" ]]; then
    discover_lib_modules
fi

if [[ "$SHOW_INTEGRATION" == "true" ]]; then
    discover_integration_tests
fi

if [[ "$SHOW_BINS" == "true" ]]; then
    discover_bin_modules
fi
