# State Hash Debug Feature - Implementation TODO

## Progress

✅ Created `src/game/state_hash.rs` module with:
- `compute_state_hash()` - Computes deterministic hash of GameState
- `strip_metadata()` - Removes ephemeral fields before hashing
- `format_hash()` - Formats hash for display (first 8 hex digits)

✅ Added `debug_state_hash: bool` field to `GameLogger`
✅ Initialized in all constructors
✅ Module exported in `src/game/mod.rs`
✅ Compiles successfully

## Remaining Work

### 1. Add Getter/Setter to GameLogger

In `src/game/logger.rs`, add after line 242 (after `should_show_choice_menu()`):

```rust
/// Enable state hash debugging
pub fn set_debug_state_hash(&mut self, enabled: bool) {
    self.debug_state_hash = enabled;
}

/// Check if state hash debugging is enabled
pub fn debug_state_hash_enabled(&self) -> bool {
    self.debug_state_hash
}
```

### 2. Modify GameState Logging Methods

In key logging functions in `src/game/state.rs` or where game actions are logged,
add hash printing before the action log. For example:

```rust
pub fn log_action(&self, message: &str) {
    // Print state hash if debug mode enabled
    if self.logger.debug_state_hash_enabled() {
        use crate::game::{compute_state_hash, format_hash};
        let hash = compute_state_hash(self);
        eprintln!("[STATE:{}] {}", format_hash(hash), message);
    }

    // Normal logging
    self.logger.normal(message);
}
```

Key places to add hash logging:
- `draw_card()` in state.rs
- `play_land()` in actions.rs
- `cast_spell()` in actions.rs
- Combat damage application
- Step/phase transitions in game_loop.rs

### 3. Add CLI Flag

In `src/main.rs`, add flag:

```rust
#[arg(long, help = "Enable state hash debugging (prints hash before each action)")]
debug_state_hash: bool,
```

Then set it:

```rust
game.logger.set_debug_state_hash(args.debug_state_hash);
```

### 4. Update Stress Test to Use Debug Mode

In `scripts/snapshot_stress_test_single.py`, add `--debug-state-hash` flag to
both normal and stop-go game runs. This will print hashes to stderr, allowing
comparison of exactly when states diverge.

## Usage Example

```bash
# Run with state hash debugging
./target/debug/mtg tui decks/royal_assassin.dck \
  --p1 random --p2 random --seed 42 \
  --debug-state-hash \
  --stop-every p1:choice:3 \
  --snapshot-output snap.json 2>hashes.txt

# Compare hashes between normal and stop-go runs
diff normal_hashes.txt stopgo_hashes.txt
```

## Expected Output Format

```
[STATE:a3f7b2c1] Turn 1 - Alice's turn
[STATE:a3f7b2c1] Alice draws Swamp (11)
[STATE:d4e8f3a2] Alice plays Swamp (11)
[STATE:d4e8f3a2] Turn 2 - Bob's turn
[STATE:d4e8f3a2] Bob draws Royal Assassin (102)
...
```

When states diverge, the hashes will differ at the exact action where divergence occurs,
making it easy to pinpoint the root cause.

## Benefits

- Pinpoint EXACTLY when game states diverge (to the specific action)
- No need to compare full JSON dumps
- Fast hash computation (~microseconds)
- Minimal performance impact (only when flag enabled)
- Works with existing logging infrastructure
