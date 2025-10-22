#!/bin/bash
# Dynamically discover and run all examples
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
PASSED=0
FAILED=0
FAILED_EXAMPLES=""

echo "=== Running $TOTAL examples ==="
echo ""

# Run each example
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
