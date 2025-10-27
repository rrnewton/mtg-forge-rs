---
title: Stop-and-go game snapshots for interactive exploration
status: open
priority: 3
issue_type: feature
created_at: "2025-10-27T00:03:50Z"
updated_at: "2025-10-27T01:25:51Z"
---

# Description

Implement replay-based game snapshots using undo log

**New Approach** (replay-based, Option 3):
Save GameState only at turn boundaries + log of intra-turn choices.

**Design**:
1. Snapshots save: GameState (at turn start) + list of choices made during turn
2. On stop: Rewind undo log to turn start, collecting choice history in reverse
3. On resume: Replay choices with buffered logging until stop point
4. Exercises both serialization AND undo/redo infrastructure

**Implementation steps**:
1. Add ChoicePoint to undo log (GameAction::ChoicePoint)
2. Track turn boundaries in undo log
3. Implement rewind-to-turn-start with choice extraction
4. Add buffered logging mode to GameLogger
5. Implement replay with log buffering until stop point
6. Update snapshot format: { game_state, turn_num, intra_turn_choices }
7. Test with royal_assassin decks

**Benefits**:
- Clean save points (turn boundaries)
- Tests undo/redo thoroughly
- Simpler than mid-execution suspension
- Deterministic replay

# Notes

## Progress Update (2025-10-27, commit dd4b0eb2)

Completed core infrastructure for replay-based snapshots:

✅ **Completed:**
1. Add ChoicePoint to undo log - Already existed in GameAction enum
2. Track turn boundaries in undo log - Already tracked via ChangeTurn
3. Implement rewind-to-turn-start with choice extraction - Done in src/undo.rs
4. Add buffered logging mode to GameLogger - Done in src/game/logger.rs
5. Update snapshot format - Done in src/game/snapshot.rs (GameSnapshot struct with save/load methods)

🔨 **In Progress:**
- CLI integration (--stop-every, --snapshot-output, --start-from flags)
- Game loop integration to trigger rewind on choice limit
- Replay logic with buffered logging

📋 **Remaining:**
- Test with royal_assassin decks
- End-to-end integration testing

**Next steps:** Integrate snapshot functionality into main.rs CLI and game loop.
