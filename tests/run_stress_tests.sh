#!/bin/bash
# Run snapshot/resume stress tests in parallel using GNU parallel
#
# Usage:
#   ./run_stress_tests.sh [deck_names...]
#   ./run_stress_tests.sh --sequential [deck_names...]
#
# Examples:
#   ./run_stress_tests.sh                                  # Run all default decks
#   ./run_stress_tests.sh royal_assassin white_aggro_4ed  # Run specific decks
#   ./run_stress_tests.sh --sequential                     # Force sequential execution
#
# If no deck names are provided, runs default set of decks.
# Deck names should be without .dck extension.

set -e  # Exit on error

# Parse command line arguments
FORCE_SEQUENTIAL=false
DECK_ARGS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --sequential)
            FORCE_SEQUENTIAL=true
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
chmod +x tests/snapshot_stress_test_single.py

# Function to run a single test case
run_test() {
    local test_case="$1"

    # Parse test case: deck_path:p1:p2
    IFS=':' read -r deck_path p1 p2 <<< "$test_case"

    # Extract deck name for display
    deck_name=$(basename "$deck_path" .dck)

    # Run the test (quietly - only errors shown)
    if ./tests/snapshot_stress_test_single.py "$deck_path" "$p1" "$p2" --quiet 2>&1; then
        echo "✓ $deck_name ($p1 vs $p2)"
        return 0
    else
        echo "✗ $deck_name ($p1 vs $p2)"
        return 1
    fi
}

export -f run_test

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
        exit 1
    fi

    echo ""
    echo "✅ All stress tests passed!"
    exit 0
fi

# Parallel execution using GNU parallel
# Run with moderate parallelism (jobs=2) since each test spawns multiple game processes
# --halt soon,fail=1: Stop as soon as one job fails
# --line-buffer: Buffer output by line (prevents interleaving)
echo "Running $TOTAL tests in parallel..."
echo ""

if printf '%s\n' "${TEST_CASES[@]}" | parallel --jobs 2 --halt soon,fail=1 --line-buffer \
    'run_test {}'; then

    echo ""
    echo "========================================"
    echo "Summary: $TOTAL/$TOTAL tests passed"
    echo "========================================"
    echo ""
    echo "✅ All stress tests passed!"
    exit 0
else
    EXIT_CODE=$?
    echo ""
    echo "========================================"
    echo "❌ Some stress tests failed!"
    echo "========================================"
    exit $EXIT_CODE
fi
