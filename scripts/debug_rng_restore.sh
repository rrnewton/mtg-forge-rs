#!/bin/bash
# Debug script to test RNG restoration with detailed logging
#
# This script runs a Random vs Heuristic game, saves a snapshot after 10 choices,
# then resumes and checks if the RNG state is correctly restored.

set -e

SEED=12345
DECK="decks/grizzly_bears.dck"
SNAPSHOT="debug_rng.snapshot"

echo "=== RNG Restoration Debug Test ==="
echo ""
echo "Step 1: Run initial game and stop after 10 choices..."
echo ""

cargo run --bin mtg -- tui "$DECK" "$DECK" \
  --p1=random --p2=heuristic \
  --seed=$SEED \
  --verbosity=3 \
  --debug-state-hash \
  --stop-every=both:choice:10 \
  --snapshot-output="$SNAPSHOT" \
  2>&1 | tee debug_initial.log

if [!  -f "$SNAPSHOT" ]; then
  echo "ERROR: Snapshot was not created!"
  exit 1
fi

echo ""
echo "Step 2: Examine RNG state in snapshot..."
echo ""

# Extract the RNG state from the snapshot JSON
jq '.p1_controller_state.Random.rng' "$SNAPSHOT" > debug_p1_rng.json || echo "No P1 RNG state"
jq '.p2_controller_state' "$SNAPSHOT" > debug_p2_state.json || echo "No P2 state"

echo "P1 (Random) RNG state in snapshot:"
cat debug_p1_rng.json
echo ""
echo "P2 controller state:"
cat debug_p2_state.json
echo ""

echo "Step 3: Resume from snapshot and continue..."
echo ""

cargo run --bin mtg -- resume "$SNAPSHOT" \
  --verbosity=3 \
  --debug-state-hash \
  2>&1 | tee debug_resume.log

echo ""
echo "=== Debug Test Complete ==="
echo ""
echo "Logs saved:"
echo "  - debug_initial.log (first 10 choices)"
echo "  - debug_resume.log (resumed game)"
echo "  - debug_p1_rng.json (P1 RNG state from snapshot)"
echo "  - debug_p2_state.json (P2 controller state)"
echo ""
echo "To compare state hashes:"
echo "  grep '\\[STATE:' debug_initial.log | tail -20"
echo "  grep '\\[STATE:' debug_resume.log | head -20"
