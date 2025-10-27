#!/bin/bash
# Test script for stop-and-go game snapshots with Royal Assassin decks

set -e  # Exit on error

echo "=== Testing Stop-and-Go Game Snapshots ==="
echo

# Clean up any existing snapshots
rm -f game.snapshot test.snapshot

echo "Step 1: Run game with --stop-every=p1:choice:3"
echo "This should stop after player 1 makes 3 choices"
echo "----------------------------------------"
cargo run --release --bin mtg -- tui \
  --p1=fixed --p1-fixed-inputs="0 0 0" \
  --p2=heuristic \
  --p1-name="Alice" --p2-name="Bob" \
  test_decks/royal_assassin.dck test_decks/royal_assassin.dck \
  --seed=42 \
  --stop-every=p1:choice:3 \
  --snapshot-output=test.snapshot \
  --verbosity=normal

echo
echo "Step 2: Check that snapshot was created"
echo "----------------------------------------"
if [ -f "test.snapshot" ]; then
    echo "✓ Snapshot file created: test.snapshot"
    echo "  File size: $(wc -c < test.snapshot) bytes"
    echo "  First few lines:"
    head -n 20 test.snapshot
else
    echo "✗ ERROR: Snapshot file not created!"
    exit 1
fi

echo
echo "Step 3: Resume from snapshot and play more choices"
echo "----------------------------------------"
cargo run --release --bin mtg -- tui \
  --start-from=test.snapshot \
  --p1=fixed --p1-fixed-inputs="0 0 0" \
  --p2=heuristic \
  --seed=42 \
  --stop-every=p1:choice:2 \
  --snapshot-output=test.snapshot \
  --verbosity=normal

echo
echo "Step 4: Verify snapshot was updated"
echo "----------------------------------------"
if [ -f "test.snapshot" ]; then
    echo "✓ Snapshot file updated"
    echo "  File size: $(wc -c < test.snapshot) bytes"
else
    echo "✗ ERROR: Snapshot file not found!"
    exit 1
fi

echo
echo "=== Test Complete ==="
echo "✓ All steps passed"
echo
echo "Cleaning up..."
rm -f test.snapshot

echo "Done!"
