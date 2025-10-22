#!/bin/bash

# Post-processing script for heaptrack profiling
# Converts raw heaptrack output and extracts top allocation sites from our source code

set -e

# Find the most recent heaptrack raw profile (try both .zst and .gz formats)
RAW_PROFILE=$(ls -t heaptrack.profile.*.raw.zst heaptrack.profile.*.raw.gz 2>/dev/null | head -n1)

if [ -z "$RAW_PROFILE" ]; then
    echo "Error: No heaptrack raw profile found (heaptrack.profile.*.raw.{zst,gz})"
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
    PROFILE_NUM=$(echo "$RAW_PROFILE" | sed 's/heaptrack.profile.\([0-9]*\).raw.zst/\1/')
elif [[ "$RAW_PROFILE" == *.gz ]]; then
    DECOMPRESS="gzip -dc"
    COMPRESS="gzip -c"
    EXT="gz"
    PROFILE_NUM=$(echo "$RAW_PROFILE" | sed 's/heaptrack.profile.\([0-9]*\).raw.gz/\1/')
else
    echo "Error: Unknown compression format for $RAW_PROFILE"
    exit 1
fi

PROCESSED_PROFILE="heaptrack.profile.${PROFILE_NUM}.${EXT}"

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
echo "=== Step 2: Extracting allocation statistics ==="
PRINT_OUTPUT="heaptrack_analysis.${PROFILE_NUM}.txt"
heaptrack_print "$PROCESSED_PROFILE" > "$PRINT_OUTPUT"
echo "✓ Full heaptrack_print output saved: $PRINT_OUTPUT"
echo ""

# Step 3: Parse and display top allocation sites in our source code
echo "=== Step 3: Top allocation sites in src/ ==="
echo ""
echo "Format: <allocations> calls with <total_bytes> at <location>"
echo "---------------------------------------------------------------"
echo ""

# Extract call stacks that have "calls with" in them and look for src/ paths
# We'll show the allocation count, the src/ file location, and the actual line of code
grep -A10 'calls with' "$PRINT_OUTPUT" | \
    grep -E '( calls with | at .*src/)' | \
    head -n 100 > /tmp/heaptrack_src_stacks.txt

# Now parse the full heaptrack_print output to find allocations from our src/ code
python3 << PYTHON_SCRIPT
import re
import sys

# Read the full heaptrack print output
with open('${PRINT_OUTPUT}', 'r') as f:
    content = f.read()

# Parse allocation blocks - format is:
# "NNNN calls to allocation functions with XXXX peak consumption from"
# followed by stack trace lines with "at path/file.rs:line"
alloc_blocks = re.split(r'(\d+) calls to allocation functions with ([\d.]+[KMGT]?B?) peak consumption from', content)

src_allocations = []

# Process blocks in groups of 3: (text_before, calls, bytes, stack_trace)
for i in range(1, len(alloc_blocks), 3):
    if i + 2 >= len(alloc_blocks):
        break

    calls = int(alloc_blocks[i])
    bytes_str = alloc_blocks[i + 1]
    stack_trace = alloc_blocks[i + 2]

    # Find all src/ references in this stack trace
    src_matches = re.findall(r'at (?:/workspace/)?src/([^:]+:\d+)', stack_trace)

    if src_matches:
        # Get the deepest (most specific) src/ location
        primary_loc = 'src/' + src_matches[0]

        # Get calling context (next 2 frames)
        context_locs = ['src/' + m for m in src_matches[1:3]]

        src_allocations.append({
            'calls': calls,
            'bytes': bytes_str,
            'location': primary_loc,
            'context': context_locs
        })

# Sort by number of calls (descending)
src_allocations.sort(key=lambda x: x['calls'], reverse=True)

# Print top 20 allocation sites
print("Top allocation sites from our code (by call count):\n")
for i, alloc in enumerate(src_allocations[:20], 1):
    print(f"{i:2d}. {alloc['calls']:>8,} calls, {alloc['bytes']:>12}")
    print(f"    └─> {alloc['location']}")

    # Show calling context
    for loc in alloc['context']:
        print(f"        ├─> {loc}")
    print()

if not src_allocations:
    print("No allocation sites found in src/ code")
    print("This might mean:")
    print("  1. Most allocations are from dependencies (good!)")
    print("  2. The profile didn't capture enough data")
    print("  3. Optimizations eliminated allocation sites")
else:
    print(f"Total allocation sites in src/ code: {len(src_allocations)}")
PYTHON_SCRIPT

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
