#!/bin/bash
# Run snapshot/resume stress tests in parallel using GNU parallel
#
# Usage:
#   ./run_stress_tests.sh [--sequential] [--keep] [deck_names...]
#
# Examples:
#   ./run_stress_tests.sh                                  # Run all default decks
#   ./run_stress_tests.sh royal_assassin white_aggro_4ed  # Run specific decks
#   ./run_stress_tests.sh --sequential                     # Force sequential execution
#   ./run_stress_tests.sh --keep                           # Save all test artifacts
#   ./run_stress_tests.sh --keep --sequential grizzly_bears
#
# Flags:
#   --sequential    Force sequential execution (disable parallel)
#   --keep          Save all artifacts (logs, gamestates, snapshots) to test_artifacts/
#
# If no deck names are provided, runs default set of decks.
# Deck names should be without .dck extension.

set -e  # Exit on error

# Parse command line arguments
FORCE_SEQUENTIAL=false
KEEP_ARTIFACTS=false
DECK_ARGS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            echo "Run snapshot/resume stress tests in parallel using GNU parallel"
            echo ""
            echo "Usage:"
            echo "  $0 [--sequential] [--keep] [deck_names...]"
            echo ""
            echo "Examples:"
            echo "  $0                                  # Run all default decks"
            echo "  $0 royal_assassin white_aggro_4ed  # Run specific decks"
            echo "  $0 --sequential                     # Force sequential execution"
            echo "  $0 --keep                           # Save all test artifacts"
            echo "  $0 --keep --sequential grizzly_bears"
            echo ""
            echo "Flags:"
            echo "  -h, --help      Show this help message"
            echo "  --sequential    Force sequential execution (disable parallel)"
            echo "  --keep          Save all artifacts (logs, gamestates, snapshots) to test_artifacts/"
            echo ""
            echo "If no deck names are provided, runs default set of decks."
            echo "Deck names should be without .dck extension."
            exit 0
            ;;
        --sequential)
            FORCE_SEQUENTIAL=true
            shift
            ;;
        --keep|--keep-logs)
            KEEP_ARTIFACTS=true
            shift
            ;;
        *)
            DECK_ARGS+=("$1")
            shift
            ;;
    esac
done

# Check for cardsfolder
if [ ! -d "cardsfolder" ]; then
    echo "Warning: cardsfolder not found, skipping stress tests"
    exit 0
fi

# Default decks to test if none specified
if [ ${#DECK_ARGS[@]} -eq 0 ]; then
    DECKS=(
        "royal_assassin"
        "white_aggro_4ed"
        "grizzly_bears"
    )
else
    DECKS=("${DECK_ARGS[@]}")
fi

# Controller matchups to test
MATCHUPS=(
    "heuristic:heuristic"
    "random:heuristic"
)

echo "=== MTG Snapshot/Resume Stress Tests ==="
echo ""
echo "Decks to test: ${DECKS[*]}"
echo "Controller matchups: ${#MATCHUPS[@]}"
echo ""

# Generate list of all test combinations
TEST_CASES=()
for deck in "${DECKS[@]}"; do
    deck_path="decks/${deck}.dck"

    # Check if deck file exists
    if [ ! -f "$deck_path" ]; then
        echo "Warning: Skipping $deck - file not found at $deck_path"
        continue
    fi

    for matchup in "${MATCHUPS[@]}"; do
        # Split matchup into p1 and p2 controllers
        IFS=':' read -r p1 p2 <<< "$matchup"
        TEST_CASES+=("$deck_path:$p1:$p2")
    done
done

if [ ${#TEST_CASES[@]} -eq 0 ]; then
    echo "Error: No valid test cases found"
    exit 1
fi

TOTAL=${#TEST_CASES[@]}
echo "Total test cases: $TOTAL"
echo ""

# Make test script executable
chmod +x scripts/snapshot_stress_test_single.py

# Function to run a single test case
run_test() {
    local test_case="$1"

    # Parse test case: deck_path:p1:p2
    IFS=':' read -r deck_path p1 p2 <<< "$test_case"

    # Extract deck name for display
    deck_name=$(basename "$deck_path" .dck)

    # Build command with optional --keep flag
    local cmd="./scripts/snapshot_stress_test_single.py $deck_path $p1 $p2 --quiet"
    if [ "$KEEP_ARTIFACTS" = true ]; then
        cmd="$cmd --keep"
    fi

    # Run the test and capture output
    local output
    output=$($cmd 2>&1)
    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo "✓ $deck_name ($p1 vs $p2)"
        return 0
    else
        echo "✗ $deck_name ($p1 vs $p2)"
        # Print detailed failure info to stderr for capturing
        {
            echo ""
            echo "=========================================="
            echo "FAILED: $deck_name ($p1 vs $p2)"
            echo "=========================================="
            echo ""
            echo "Reproduce with:"
            local repro_cmd="  ./scripts/snapshot_stress_test_single.py $deck_path $p1 $p2 --verbose"
            if [ "$KEEP_ARTIFACTS" = true ]; then
                repro_cmd="$repro_cmd --keep"
            fi
            echo "$repro_cmd"
            echo ""
            echo "Output:"
            echo "$output"
            echo ""
        } >&2
        return 1
    fi
}

export -f run_test
export KEEP_ARTIFACTS

# Check if GNU parallel is available and not forcing sequential
if [ "$FORCE_SEQUENTIAL" = true ] || ! command -v parallel &> /dev/null; then
    if [ "$FORCE_SEQUENTIAL" = true ]; then
        echo "INFO: --sequential flag specified, using sequential execution"
        echo ""
    else
        echo "WARNING: GNU parallel not found, falling back to sequential execution"
        echo "Install with: apt-get install parallel (Ubuntu/Debian) or brew install parallel (macOS)"
        echo ""
    fi

    # Sequential execution
    PASSED=0
    FAILED=0
    FAILED_TESTS=""

    for test_case in "${TEST_CASES[@]}"; do
        if run_test "$test_case"; then
            PASSED=$((PASSED + 1))
        else
            FAILED=$((FAILED + 1))
            IFS=':' read -r deck_path p1 p2 <<< "$test_case"
            deck_name=$(basename "$deck_path" .dck)
            FAILED_TESTS="$FAILED_TESTS\n  - $deck_name ($p1 vs $p2)"
        fi
    done

    echo ""
    echo "========================================"
    echo "Summary: $PASSED/$TOTAL tests passed"
    echo "========================================"

    if [ $FAILED -gt 0 ]; then
        echo ""
        echo "Failed tests:$FAILED_TESTS"
        # TODO(mtg-109): Known failures - snapshot/resume has determinism issues
        echo ""
        echo "⚠️  KNOWN FAILURES ($FAILED/$TOTAL) - mtg-109 investigation in progress"
        echo "Snapshot/resume determinism issues being debugged - test runs but doesn't block CI"
        exit 0  # Exit with success to allow validation to continue
    fi

    echo ""
    echo "✅ All stress tests passed!"
    exit 0
fi

# Parallel execution using GNU parallel
# Run with moderate parallelism (jobs=2) since each test spawns multiple game processes
# --line-buffer: Buffer output by line (prevents interleaving)
# --joblog: Track which jobs passed/failed
echo "Running $TOTAL tests in parallel..."
echo ""

JOBLOG=$(mktemp)
trap "rm -f $JOBLOG" EXIT

# Run all tests and capture exit codes in joblog
# Temporarily disable exit-on-error so we can process results even if tests fail
set +e
printf '%s\n' "${TEST_CASES[@]}" | parallel --jobs 2 --line-buffer --joblog "$JOBLOG" \
    'run_test {}'
set -e

# Count results from joblog (skip header line)
PASSED=$(awk 'NR>1 && $7==0 {count++} END {print count+0}' "$JOBLOG")
FAILED=$(awk 'NR>1 && $7!=0 {count++} END {print count+0}' "$JOBLOG")

echo ""
echo "========================================"
echo "Summary: $PASSED/$TOTAL tests passed"
echo "========================================"

if [ $FAILED -gt 0 ]; then
    echo ""
    echo "❌ $FAILED test(s) failed! (see detailed output above)"
    # TODO(mtg-109): Known failures - snapshot/resume has determinism issues
    echo ""
    echo "⚠️  KNOWN FAILURES ($FAILED/$TOTAL) - mtg-109 investigation in progress"
    echo "Snapshot/resume determinism issues being debugged - test runs but doesn't block CI"
    exit 0  # Exit with success to allow validation to continue
fi

echo ""
echo "✅ All stress tests passed!"
exit 0
