#!/bin/bash

# Post-processing script for heaptrack profiling
# Converts raw heaptrack output and displays top allocation sites

set -e

# Create experiment_results directory if it doesn't exist
mkdir -p experiment_results

# Find the most recent heaptrack raw profile (try both .zst and .gz formats)
RAW_PROFILE=$(ls -t experiment_results/heaptrack.profile.*.raw.zst experiment_results/heaptrack.profile.*.raw.gz 2>/dev/null | head -n1)

if [ -z "$RAW_PROFILE" ]; then
    echo "Error: No heaptrack raw profile found (experiment_results/heaptrack.profile.*.raw.{zst,gz})"
    echo "Run 'make heapprofile' first to generate a profile"
    exit 1
fi

echo "Found raw profile: $RAW_PROFILE"
echo ""

# Determine compression format and set appropriate commands
if [[ "$RAW_PROFILE" == *.zst ]]; then
    DECOMPRESS="zstd -dc"
    COMPRESS="zstd -c"
    EXT="zst"
    PROFILE_NUM=$(echo "$RAW_PROFILE" | sed 's/.*heaptrack.profile.\([0-9]*\).raw.zst/\1/')
elif [[ "$RAW_PROFILE" == *.gz ]]; then
    DECOMPRESS="gzip -dc"
    COMPRESS="gzip -c"
    EXT="gz"
    PROFILE_NUM=$(echo "$RAW_PROFILE" | sed 's/.*heaptrack.profile.\([0-9]*\).raw.gz/\1/')
else
    echo "Error: Unknown compression format for $RAW_PROFILE"
    exit 1
fi

PROCESSED_PROFILE="experiment_results/heaptrack.profile.${PROFILE_NUM}.${EXT}"

# Find heaptrack_interpret - try common locations
HEAPTRACK_INTERPRET=""
for path in /usr/lib64/heaptrack/libexec/heaptrack_interpret /usr/lib/heaptrack/libexec/heaptrack_interpret; do
    if [ -x "$path" ]; then
        HEAPTRACK_INTERPRET="$path"
        break
    fi
done

if [ -z "$HEAPTRACK_INTERPRET" ]; then
    echo "Error: Cannot find heaptrack_interpret"
    exit 1
fi

# Step 1: Process raw => processed (if not already done)
if [ ! -f "$PROCESSED_PROFILE" ]; then
    echo "=== Step 1: Processing raw profile ==="
    echo "Running: $DECOMPRESS < $RAW_PROFILE | $HEAPTRACK_INTERPRET | $COMPRESS > $PROCESSED_PROFILE"
    $DECOMPRESS < "$RAW_PROFILE" | "$HEAPTRACK_INTERPRET" | $COMPRESS > "$PROCESSED_PROFILE"
    echo "✓ Processed profile saved: $PROCESSED_PROFILE"
    echo ""
else
    echo "✓ Processed profile already exists: $PROCESSED_PROFILE"
    echo ""
fi

# Step 2: Run heaptrack_print to get allocation statistics
echo "=== Step 2: Top allocation sites with debug symbols ==="
PRINT_OUTPUT="experiment_results/heaptrack_analysis.${PROFILE_NUM}.txt"
heaptrack_print "$PROCESSED_PROFILE" > "$PRINT_OUTPUT"
echo ""

# Display top allocations involving our source code (src/)
# heaptrack_print output already includes function names if debug symbols are present
echo "Top allocation sites from src/ (showing first 3 stack frames):"
echo "---------------------------------------------------------------"
grep -A 3 "calls.*consumption from" "$PRINT_OUTPUT" | grep -B 3 "src/" | head -100
echo ""
echo "---------------------------------------------------------------"
echo ""
echo "To view full details:"
echo "  cat $PRINT_OUTPUT"
echo ""
echo "To view graphically:"
echo "  heaptrack_gui $PROCESSED_PROFILE"
echo ""
echo "Files generated:"
echo "  - $PROCESSED_PROFILE (processed profile for heaptrack_gui)"
echo "  - $PRINT_OUTPUT (full text analysis)"
echo ""
