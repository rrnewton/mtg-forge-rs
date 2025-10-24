#!/bin/bash
# Dynamically discover and run all examples in parallel using GNU parallel
# Exits with success only if all examples succeed

set -e  # Exit on error

echo "=== Discovering available examples ==="
# Get list of examples by parsing cargo output
EXAMPLES=$(cargo run --example 2>&1 | grep -A 1000 "Available examples:" | tail -n +2 | sed 's/^[[:space:]]*//' | grep -v '^$')

if [ -z "$EXAMPLES" ]; then
    echo "ERROR: No examples found!"
    exit 1
fi

echo "Found examples:"
echo "$EXAMPLES"
echo ""

# Count examples
TOTAL=$(echo "$EXAMPLES" | wc -l)

echo "=== Running $TOTAL examples in parallel ==="
echo ""

# Check if GNU parallel is available
if ! command -v parallel &> /dev/null; then
    echo "WARNING: GNU parallel not found, falling back to sequential execution"
    echo "Install with: apt-get install parallel (Ubuntu/Debian) or brew install parallel (macOS)"
    echo ""

    # Fallback to sequential execution
    PASSED=0
    FAILED=0
    FAILED_EXAMPLES=""

    while IFS= read -r example; do
        echo "----------------------------------------"
        echo "Running example: $example"
        echo "----------------------------------------"

        if cargo run --example "$example" 2>&1; then
            echo ""
            echo "✅ $example: PASSED"
            PASSED=$((PASSED + 1))
        else
            echo ""
            echo "❌ $example: FAILED"
            FAILED=$((FAILED + 1))
            FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $example"
        fi
        echo ""
    done <<< "$EXAMPLES"

    echo "========================================"
    echo "Summary: $PASSED/$TOTAL examples passed"
    echo "========================================"

    if [ $FAILED -gt 0 ]; then
        echo ""
        echo "Failed examples:$FAILED_EXAMPLES"
        exit 1
    fi

    echo ""
    echo "✅ All examples passed!"
    exit 0
fi

# Use GNU parallel for parallel execution
# -j 4: Run 4 jobs in parallel (tunable based on system)
# --halt soon,fail=1: Stop as soon as one job fails
# --line-buffer: Buffer output by line (prevents interleaving)
# --tagstring: Prefix output with example name
# --results /tmp/example_results: Store results for analysis

# Create temp directory for results
RESULTS_DIR=$(mktemp -d)
trap "rm -rf $RESULTS_DIR" EXIT

# Run function for each example
run_example() {
    local example="$1"
    local output_file="$2"

    echo "----------------------------------------" > "$output_file"
    echo "Running example: $example" >> "$output_file"
    echo "----------------------------------------" >> "$output_file"

    if cargo run --example "$example" >> "$output_file" 2>&1; then
        echo "" >> "$output_file"
        echo "✅ $example: PASSED" >> "$output_file"
        return 0
    else
        echo "" >> "$output_file"
        echo "❌ $example: FAILED" >> "$output_file"
        return 1
    fi
}

export -f run_example

# Run examples in parallel with output buffering
# Use --jobs 4 for moderate parallelism (adjust based on testing)
if echo "$EXAMPLES" | parallel --jobs 4 --halt soon,fail=1 --line-buffer \
    'run_example {} '"$RESULTS_DIR"'/{}.log && cat '"$RESULTS_DIR"'/{}.log || (cat '"$RESULTS_DIR"'/{}.log; exit 1)'; then

    echo ""
    echo "========================================"
    echo "Summary: $TOTAL/$TOTAL examples passed"
    echo "========================================"
    echo ""
    echo "✅ All examples passed!"
    exit 0
else
    EXIT_CODE=$?
    echo ""
    echo "========================================"
    echo "❌ Some examples failed!"
    echo "========================================"
    echo ""

    # List failed examples
    FAILED_COUNT=0
    for example in $EXAMPLES; do
        if [ -f "$RESULTS_DIR/$example.log" ]; then
            if grep -q "FAILED" "$RESULTS_DIR/$example.log"; then
                echo "  - $example"
                FAILED_COUNT=$((FAILED_COUNT + 1))
            fi
        fi
    done

    PASSED=$((TOTAL - FAILED_COUNT))
    echo ""
    echo "Summary: $PASSED/$TOTAL examples passed"
    exit $EXIT_CODE
fi
