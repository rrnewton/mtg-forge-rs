# State Hash Debug - Remaining Integration Work

## Completed ✅
- State hash computation module (`src/game/state_hash.rs`)
- GameLogger field and getter/setter methods
- `GameState::debug_log_state_hash()` helper method
- Everything compiles

## Next Steps 🚧

### 1. Add CLI Flag (5 min)

In `src/main.rs`:
```rust
#[arg(long, help = "Enable state hash debugging")]
debug_state_hash: bool,

// After game init:
if args.debug_state_hash {
    game.logger.set_debug_state_hash(true);
}
```

### 2. Add Logging Hooks (15 min)

Call `game.debug_log_state_hash(msg)` before key actions:
- `draw_card()` - before logging the draw
- `play_land()` / `cast_spell()` - before logging the action
- Turn headers in `game_loop.rs`
- Step changes
- Combat damage

### 3. Test (5 min)

```bash
cargo build
./target/debug/mtg tui decks/royal_assassin.dck \
  --p1 random --p2 random --seed 42 \
  --debug-state-hash 2> hashes.txt

# Output should show:
# [STATE:a3f7b2c1] Turn 1 - Alice's turn
# [STATE:a3f7b2c1] Alice draws Swamp (11)
# [STATE:d4e8f3a2] Alice plays Swamp (11)
```

## Usage for Debugging Divergence

```bash
# Normal mode
./target/debug/mtg tui decks/royal_assassin.dck \
  --p1 random --p2 random --seed 42 \
  --debug-state-hash 2> normal.txt

# Stop-go mode
./target/debug/mtg tui decks/royal_assassin.dck \
  --p1 random --p2 random --seed 42 \
  --stop-every p1:choice:3 \
  --snapshot-output /tmp/s.json \
  --debug-state-hash 2> stopgo.txt

# Find divergence
diff normal.txt stopgo.txt | head
```

Total remaining work: ~25 minutes
