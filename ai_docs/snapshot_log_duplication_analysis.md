# Snapshot Stress Test Log Duplication Analysis

**Date**: 2025-10-29_#167
**Status**: Pre-existing issue since at least commit 951b028
**Severity**: LOW - Cosmetic logging issue, game logic is correct

## Problem Summary

When running the snapshot stress test, the stop-and-go logs contain duplicate turn headers and draw statements compared to the normal logs. GameStates match exactly (✓), indicating this is purely a logging presentation issue, not a game logic bug.

## Example

Normal log:
```
Turn 2 - Bob's turn
Life: 20
Life: 20
Bob draws Royal Assassin (113)
Bob plays Swamp (63)
```

Stop-go log:
```
Turn 2 - Bob's turn
Life: 20
Life: 20
Bob draws Royal Assassin (113)
Turn 2 - Bob's turn    <-- DUPLICATE
Life: 20
Life: 20
Bob draws Royal Assassin (113)  <-- DUPLICATE
Bob plays Swamp (63)
```

## Root Cause

The issue occurs in the snapshot/resume workflow:

1. **Original Run**: Game executes normally
   - `run_turn()` prints "Turn 2 - Bob's turn" (game_loop.rs:567)
   - `draw_step()` prints "Bob draws Royal Assassin" (game_loop.rs:1044)
   - Both print immediately to stdout

2. **Save Snapshot**: `rewind_to_turn_start()` is called (undo.rs:165-210)
   - Pops actions from undo log until finding `ChangeTurn`
   - Undoes game state changes (cards, life, mana, etc.)
   - Keeps `ChangeTurn` action on the log (game state now at turn boundary)
   - **BUT: stdout has already been written and cannot be rewound**

3. **Resume**: Game replays from turn boundary
   - `run_turn()` prints "Turn 2 - Bob's turn" AGAIN
   - `draw_step()` prints "Bob draws Royal Assassin" AGAIN
   - Result: duplicate log entries

The fundamental problem: `rewind_to_turn_start()` rewinds the game state but not the stdout log.

## Key Files

- **undo.rs:165-210** - `rewind_to_turn_start()` method that rewinds game state
- **game_loop.rs:567** - Turn header printing in `run_turn()`
- **game_loop.rs:1044** - Draw statement printing in `draw_step()`
- **main.rs:288-316** - Snapshot loading code
- **tests/snapshot_stress_test.py** - Test that detects this issue

## Proposed Solutions

### Option 1: Track Logged Actions in GameState (Recommended)

Add a field to `TurnStructure` to track which turn events have been logged:

```rust
pub struct TurnStructure {
    // ... existing fields ...

    /// Track what turn events have been logged to prevent duplicate output after resume
    logged_events: HashSet<LoggedEvent>,
}

enum LoggedEvent {
    TurnHeader,
    DrawStep,
    // ... other events ...
}
```

Then in `run_turn()` and `draw_step()`:
```rust
if !self.game.turn.logged_events.contains(&LoggedEvent::TurnHeader) {
    println!("Turn {} - {}'s turn", ...);
    self.game.turn.logged_events.insert(LoggedEvent::TurnHeader);
}
```

This tracked state would be serialized in snapshots and cleared at turn boundaries.

**Pros**:
- Clean separation of concerns
- Works for all logging scenarios
- State is preserved in snapshots

**Cons**:
- Adds state tracking overhead
- Requires refactoring multiple logging sites

### Option 2: Suppress Logging on Resume

Pass a "suppress_initial_logs" flag when resuming from snapshot:

```rust
// In main.rs when resuming
let mut game_loop = GameLoop::new(&mut game)
    .with_verbosity(verbosity)
    .suppress_resume_logs(snapshot_turn_number.is_some());
```

Then in `run_turn()` and `draw_step()`:
```rust
if !self.suppress_resume_logs {
    println!("Turn {} - {}'s turn", ...);
}
self.suppress_resume_logs = false; // Clear after first turn
```

**Pros**:
- Simpler implementation
- No state tracking needed

**Cons**:
- Hacky - relies on timing
- Fragile if replay logic changes
- Only suppresses logs for one turn

### Option 3: Capture and Replay Logs (Event-Driven)

Redesign logging to be event-driven rather than immediate stdout:

```rust
pub enum LogEvent {
    TurnHeader { turn: u32, player: String },
    DrawCard { player: String, card: String },
    // ... other events ...
}

// Log events to a buffer instead of stdout
self.log_buffer.push(LogEvent::TurnHeader { ... });

// Only print to stdout if not captured
if !self.is_capturing {
    self.flush_to_stdout();
}
```

Then track which events have been printed in the snapshot.

**Pros**:
- Most robust solution
- Enables log replay/analysis
- Clean event-driven architecture

**Cons**:
- Major refactoring required
- Changes entire logging architecture
- Overkill for this cosmetic issue

## Recommendation

**For now**: Document the issue as a known limitation in the stress test.

**For later**: Implement Option 1 (track logged events) if we need perfect log matching for testing or if this causes user confusion. The effort is moderate and the solution is clean.

**Not recommended**: Option 3 is too much work for a cosmetic issue.

## Test Impact

The snapshot stress test currently:
- ✓ Verifies GameStates match exactly (PASSING)
- ✗ Expects logs to match exactly (FAILING due to duplicates)

We can either:
1. Fix the logging (Options 1-3 above)
2. Update the test to ignore/skip duplicate turn headers and draw statements
3. Document as known limitation

## Related Code

See also:
- `logger.rs` - GameLogger implementation (already supports capture mode)
- `game_loop.rs:458` - `save_snapshot_and_exit()` method
- `snapshot.rs` - GameSnapshot serialization
