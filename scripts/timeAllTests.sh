#!/bin/bash
#
# timeAllTests.sh - Time all test suites individually using hyperfine
#
# This script uses discoverTests.sh to find all test suites and times each one
# individually using hyperfine. Results are saved to a timestamped file.
#
# Usage:
#   ./scripts/timeAllTests.sh [OPTIONS]
#
# Options:
#   --lib           Only time library test modules
#   --integration   Only time integration tests
#   --warmup N      Number of warmup runs (default: 1)
#   --runs N        Number of benchmark runs (default: 3)
#   --output FILE   Output file (default: /tmp/test-timing-TIMESTAMP.txt)
#   --help          Show this help message
#
# Examples:
#   # Time all lib tests with default settings
#   ./scripts/timeAllTests.sh --lib
#
#   # Time integration tests with 5 runs each
#   ./scripts/timeAllTests.sh --integration --runs 5
#
#   # Time all tests with custom output file
#   ./scripts/timeAllTests.sh --output benchmark-results.txt

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default options
TEST_TYPE="--all"
WARMUP=1
RUNS=3
OUTPUT_FILE="/tmp/test-timing-$(date +%Y%m%d-%H%M%S).txt"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --lib)
            TEST_TYPE="--lib"
            shift
            ;;
        --integration)
            TEST_TYPE="--integration"
            shift
            ;;
        --warmup)
            WARMUP="$2"
            shift 2
            ;;
        --runs)
            RUNS="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --help|-h)
            sed -n '/^# timeAllTests.sh/,/^$/p' "$0" | grep "^#" | grep -v "^#!/" | sed 's/^# //' | sed 's/^#//'
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
done

# Check if hyperfine is installed
if ! command -v hyperfine &> /dev/null; then
    echo "Error: hyperfine is not installed" >&2
    echo "Install with: apt-get install hyperfine" >&2
    exit 1
fi

# Get test suites
echo "Discovering test suites..."
SUITES=$("$SCRIPT_DIR/discoverTests.sh" $TEST_TYPE)
SUITE_COUNT=$(echo "$SUITES" | wc -l)

echo "Found $SUITE_COUNT test suites"
echo "Warmup: $WARMUP, Runs: $RUNS"
echo "Output: $OUTPUT_FILE"
echo ""

# Prepare output file
{
    echo "=========================================="
    echo "Test Timing Report"
    echo "=========================================="
    echo "Date: $(date)"
    echo "Test type: $TEST_TYPE"
    echo "Warmup runs: $WARMUP"
    echo "Benchmark runs: $RUNS"
    echo "Total suites: $SUITE_COUNT"
    echo ""
} > "$OUTPUT_FILE"

# Time each suite
SUITE_NUM=0
for suite in $SUITES; do
    SUITE_NUM=$((SUITE_NUM + 1))
    echo "[$SUITE_NUM/$SUITE_COUNT] Timing: $suite"

    # Determine cargo test command based on suite type
    if echo "$suite" | grep -q "::"; then
        # Library test module
        CMD="cargo test --lib $suite -- --test-threads=1"
    else
        # Integration test
        CMD="cargo test --test $suite -- --test-threads=1"
    fi

    # Run hyperfine and append to output
    {
        echo "=========================================="
        echo "Suite: $suite"
        echo "=========================================="
        hyperfine --warmup "$WARMUP" --runs "$RUNS" "$CMD" 2>&1
        echo ""
    } >> "$OUTPUT_FILE"
done

echo ""
echo "=========================================="
echo "Timing complete!"
echo "Results saved to: $OUTPUT_FILE"
echo "=========================================="
echo ""
echo "Summary (from fastest to slowest):"
grep "Time (mean" "$OUTPUT_FILE" | \
    awk '{print $5, $6, $7}' | \
    sed 's/±/ ±/' | \
    paste -d' ' - <(grep "^Suite:" "$OUTPUT_FILE" | sed 's/Suite: //') | \
    sort -n | \
    head -10
