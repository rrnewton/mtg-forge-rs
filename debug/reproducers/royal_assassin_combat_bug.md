# Royal Assassin Combat Bug Reproducer

## Bug Description
When Royal Assassin destroys an attacking Hypnotic Specter during the declare attackers step, the destroyed creature incorrectly still deals combat damage.

**MTG Rule 510.1c**: "A creature or planeswalker that's no longer on the battlefield doesn't deal combat damage."

## Reproducer Command

```bash
cargo run --release --bin mtg -- tui \
  --start-state puzzles/royal_assassin_combat_bug.pzl \
  --p1=fixed --p1-fixed-inputs="1" \
  --p2=zero \
  --verbosity=verbose
```

## Expected Behavior

```
--- Declare Attackers Step ---
  Player 1 declares Hypnotic Specter (6) (2/2) as attacker
  Royal Assassin activates ability: Destroy target tapped creature.
  Hypnotic Specter (6) is destroyed
--- Declare Blockers Step ---
--- Combat Damage Step ---
  (No damage dealt - Hypnotic Specter is no longer on battlefield)
```

## Actual Behavior

```
--- Declare Attackers Step ---
  Player 1 declares Hypnotic Specter (6) (2/2) as attacker
  Royal Assassin activates ability: Destroy target tapped creature.
--- Declare Blockers Step ---
--- Combat Damage Step ---
  Hypnotic Specter (6) deals 2 damage to Player 2
```

The Hypnotic Specter deals 2 damage even though it was destroyed before the combat damage step.

## Root Cause Analysis

The bug likely occurs because:
1. Attackers are declared and stored in combat state
2. Royal Assassin's activated ability destroys the tapped attacker
3. The combat damage step still references the stored attackers list
4. The code doesn't check if attackers are still on the battlefield before dealing damage

## Fix Requirements

Before dealing combat damage in the combat damage step, the code must verify that each attacking and blocking creature is still on the battlefield. Creatures that have been destroyed, exiled, or otherwise removed should not deal combat damage.

## Puzzle File

The puzzle file is located at `/workspace/puzzles/royal_assassin_combat_bug.pzl` and sets up:
- Player 1: 3 Swamps + Hypnotic Specter (2/2)
- Player 2: 3 Swamps + Royal Assassin (1/1)
- Turn 3, Player 1's Main Phase

The fixed input "1" causes Player 1 to attack with the Hypnotic Specter (the only creature), which triggers Royal Assassin's ability.
