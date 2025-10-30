#!/usr/bin/env bash
#
# Royal Assassin Combat Bug Reproducer
#
# BUG DESCRIPTION:
# When Royal Assassin destroys an attacking Hypnotic Specter during the declare
# attackers step, the destroyed creature incorrectly still deals combat damage.
#
# MTG RULE 510.1c:
# "A creature or planeswalker that's no longer on the battlefield doesn't deal
# combat damage."
#
# EXPECTED BEHAVIOR:
# - Player 1 declares Hypnotic Specter (2/2) as attacker
# - Royal Assassin activates: Destroy target tapped creature
# - Hypnotic Specter is destroyed and moved to graveyard
# - Combat damage step: NO damage dealt (creature not on battlefield)
#
# ACTUAL BEHAVIOR:
# - Player 1 declares Hypnotic Specter (2/2) as attacker
# - Royal Assassin activates: Destroy target tapped creature
# - Combat damage step: Hypnotic Specter deals 2 damage to Player 2 (BUG!)
#
# ROOT CAUSE:
# The combat damage step doesn't verify that attackers/blockers are still on
# the battlefield before dealing damage. Creatures removed during combat should
# not deal damage.
#

set -euo pipefail

# Navigate to workspace root (script can be run from anywhere)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$WORKSPACE_ROOT"

echo "========================================="
echo "Royal Assassin Combat Bug Reproducer"
echo "========================================="
echo ""
echo "This reproducer demonstrates a bug where a creature destroyed"
echo "during the declare attackers step still deals combat damage."
echo ""
echo "Setup:"
echo "  - Player 1: Hypnotic Specter (2/2) + 3 Swamps"
echo "  - Player 2: Royal Assassin (1/1) + 3 Swamps"
echo ""
echo "Expected: Hypnotic Specter destroyed, NO damage dealt"
echo "Actual:   Hypnotic Specter destroyed, STILL deals 2 damage (BUG!)"
echo ""
echo "Running reproducer..."
echo "========================================="
echo ""

cargo run --release --bin mtg -- tui \
  --start-state debug/reproducers/royal_assassin_combat_bug.pzl \
  --p1=fixed --p1-fixed-inputs="1" \
  --p2=zero \
  --verbosity=verbose

echo ""
echo "========================================="
echo "Look for the bug in the output above:"
echo "  1. 'Player 1 declares Hypnotic Specter (6) (2/2) as attacker'"
echo "  2. 'Royal Assassin activates ability: Destroy target tapped creature.'"
echo "  3. 'Hypnotic Specter (6) deals 2 damage to Player 2' <- BUG!"
echo ""
echo "The creature should NOT deal damage after being destroyed."
echo "========================================="
