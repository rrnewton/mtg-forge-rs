# Snapshot/Resume Determinism Issues

## Date: 2025-10-27 (Commit depth: TBD)

## Summary

Implemented GameState serialization comparison in the snapshot stress test (mtg-89) and discovered significant determinism issues when resuming games from snapshots.

## Test Setup

- **Test Type**: Random vs Heuristic controller matchups
- **Decks Tested**: Royal Assassin, White Aggro 4ED, Grizzly Bears
- **Methodology**:
  1. Run normal game with random P1 and heuristic P2
  2. Extract P1's random choices from verbose log
  3. Run stop-and-go game with 5 snapshot/resume cycles, replaying P1's choices with fixed controller
  4. Compare both game action logs AND final GameState serialization

## Results: All Tests Failed

### Royal Assassin Deck
- **Normal game**: 571 actions logged
- **Stop-and-go**: 562 actions logged
- **Log differences**: 1,449 line differences
- **GameState differences**: 15 line differences in JSON serialization

### White Aggro 4ED Deck
- **Normal game**: 130 actions logged
- **Stop-and-go**: 536 actions logged (4x more actions!)
- **Log differences**: 652 line differences
- **GameState differences**: 33 line differences

### Grizzly Bears Deck
- **Normal game**: 89 actions logged
- **Stop-and-go**: 153 actions logged
- **Log differences**: 235 line differences
- **GameState differences**: 24 line differences

## Types of Differences Observed

### 1. Card ID Mismatches (Most Common)
**Normal game:**
```
Alice casts Grizzly Bears (52) (putting on stack)
```

**Stop-and-go:**
```
Alice casts Grizzly Bears (34) (putting on stack)
```

The GameState JSON shows different card IDs in the same positions:
```
Normal:    25, 73, 9, 76, 3, ...
Stop-go:   9, 25, 73, 3, 76, ...
```

**Analysis**: This suggests that card IDs are being generated differently when resuming from snapshots. This could be because:
- The card ID generator counter isn't being preserved in snapshots
- Cards are being created/drawn in a different order during replay
- The EntityStore's next ID counter isn't serialized/restored properly

### 2. Turn Number Drift
Games are getting out of sync by entire turns. Example from Grizzly Bears:
- Normal game reaches Turn 6 when stop-and-go is at Turn 7
- By the end, normal game has Turn 13 when stop-and-go has Turn 26

**Analysis**: This indicates that snapshot/resume is causing actions to be duplicated or skipped, leading to turn structure corruption.

### 3. Action Order Changes
**Normal game:**
```
Alice casts Grizzly Bears (52)
Turn 6 - Bob's turn
```

**Stop-and-go:**
```
Alice plays Forest (4)
Alice casts Grizzly Bears (34)
>>> HEURISTIC: chose not to block...
Grizzly Bears (55) deals 2 damage to Bob
Turn 7 - Bob's turn
```

**Analysis**: The stop-and-go game has extra actions that don't appear in the normal game. This suggests:
- Intra-turn replay is adding duplicate actions
- Priority passes are being replayed incorrectly
- The turn structure is being corrupted during snapshot/resume

### 4. Game Length Dramatically Different
White Aggro 4ED shows the most extreme case:
- Normal game: 130 actions
- Stop-and-go: 536 actions (over 4x longer!)

**Analysis**: This indicates catastrophic failure of determinism - the games are taking completely different paths after snapshot/resume.

## Root Causes (Hypotheses)

### High Priority Issues

1. **Card ID Generation Not Preserved**
   - The `EntityStore<Card>` has a `next_id` counter that tracks the next card ID
   - This counter is likely NOT being serialized in GameState
   - When resuming, it starts from 0 or some wrong value
   - This causes new cards to get IDs that conflict with existing cards

2. **Intra-Turn Replay Logic Corrupted**
   - The `GameSnapshot.intra_turn_choices` are meant to replay actions within a turn
   - Something about this replay is causing duplicate actions or corrupted state
   - The turn structure gets out of sync

3. **RNG State Not Fully Captured**
   - Even with fixed controllers replaying choices, something is using RNG incorrectly
   - Library shuffling or card drawing might be using RNG during snapshot/resume
   - The `game.rng_seed` field might not be sufficient to restore RNG state mid-game

### Medium Priority Issues

4. **Priority Pass Replay**
   - Fixed controller replays choices, but priority passes might not be tracked correctly
   - This could cause the game to skip or duplicate priority windows

5. **Game Phase/Step State**
   - The turn structure (phase, step) might not be preserved correctly
   - Resume might be starting in the wrong phase

## Recommendations

### Immediate Investigation Needed

1. **Check EntityStore Serialization**
   - Review `src/core/entity_store.rs` to see if `next_id` is being serialized
   - If not, add it to the `#[derive(Serialize, Deserialize)]` with explicit field handling
   - Test that card IDs are preserved across snapshot/resume

2. **Review GameSnapshot Replay Logic**
   - Check `src/game/game_loop.rs` for how `intra_turn_choices` are replayed
   - Verify that choice replay doesn't duplicate actions
   - Add logging to show exactly what choices are being replayed

3. **Add Determinism Debug Mode**
   - Create a `--debug-determinism` flag that logs:
     - Every RNG call and its result
     - Every card ID generation
     - Every choice made and replayed
     - Every turn/phase transition
   - Run stress test with this flag to pinpoint exact divergence point

### Long-term Solutions

1. **Unit Tests for Snapshot Determinism**
   - Create minimal test cases (e.g., 2-turn games)
   - Test that snapshot at turn 1 → resume → end matches normal game
   - Test with different snapshot points within the same turn

2. **GameState Diffing Tool**
   - Create a script to diff two GameState JSON files intelligently
   - Highlight which specific fields differ (card IDs, turn state, zones, etc.)
   - Use this to quickly diagnose snapshot issues

3. **Snapshot Validation**
   - Add a `GameState::validate()` method that checks for:
     - Duplicate card IDs
     - Cards in multiple zones
     - Invalid turn structure
   - Call this after every snapshot load

## Test Artifacts

Filtered log files saved in `test_logs/`:
- `Royal_Assassin_randomvheuristic_seed42_normal.log`
- `Royal_Assassin_randomvheuristic_seed42_stopgo.log`
- `White_Aggro_4ED_randomvheuristic_seed42_normal.log`
- `White_Aggro_4ED_randomvheuristic_seed42_stopgo.log`
- `Grizzly_Bears_randomvheuristic_seed42_normal.log`
- `Grizzly_Bears_randomvheuristic_seed42_stopgo.log`

These logs show only the game actions (not snapshot messages) for direct comparison.

## Next Steps

1. ✅ Implement GameState serialization comparison (COMPLETED)
2. 🔄 Document determinism issues (THIS DOCUMENT - IN PROGRESS)
3. ⏳ Investigate EntityStore next_id preservation
4. ⏳ Fix card ID generation across snapshots
5. ⏳ Re-run stress tests to verify fixes
6. ⏳ Make stress test run NUM_REPLAYS=3 per deck
7. ⏳ Split into per-deck script and parallel shell runner
8. ⏳ Investigate heuristic AI issues (losing to random, games ending by decking)

## Conclusion

The snapshot/resume mechanism has fundamental determinism issues that need to be fixed before it can be considered reliable. The primary issue appears to be card ID generation not being preserved, which cascades into numerous other problems. This is a **critical bug** that prevents reliable stop-and-resume gameplay.

The test infrastructure is now in place to verify fixes - we can re-run the stress tests after each fix attempt to measure progress.
