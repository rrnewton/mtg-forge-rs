#!/bin/bash

set -e  # Exit on error

# Check arguments
if [ $# -ne 1 ]; then
    echo "Usage: $0 ITERS"
    exit 1
fi

ITERS=$1

# Find unused log filename
XY=1
while [ -f "./logs/claude_workstream$(printf '%02d' $XY).jsonl" ]; do
    XY=$((XY + 1))
done

LOG="./logs/claude_workstream$(printf '%02d' $XY).jsonl"
echo "Using log file: $LOG"

# Ensure logs directory exists
mkdir -p ./logs

# Run iterations
for ((i=1; i<=ITERS; i++)); do
    echo "=== Iteration $i of $ITERS ==="

    # Run claude command, tee to log, and extract results
    # Remove control characters (except newline) before passing to jq to avoid parse errors
    time claude --dangerously-skip-permissions --verbose --output-format stream-json -c -p "$(cat generic_forward_progress_task.txt)" | \
        tee -a "$LOG" | \
        perl -pe 's/[\x00-\x09\x0b-\x1f]//g' | \
	jq  -r 'select (.type == "assistant" or .type == "result") | [.message.content.[0].text, .result]'

    # Check for error.txt
    if [ -f error.txt ]; then
        echo "Error detected in error.txt:"
        cat error.txt
        exit 1
    fi

    echo "Completed iteration $i"
done

echo "Successfully completed all $ITERS iterations"
exit 0
