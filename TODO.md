# MTG Forge Rust - TODO

## Current Status

**Latest Commit:** [pending] - Implement async card loading with eager and lazy modes

**Tests:** 95 passing ✅ (80 unit + 10 card loading + 5 e2e)

---

## ✅ Phase 3 Started: Gameplay Expansion

### Completed Features:
- ✅ **TUI support - Basic implementation**
  * Main binary `mtg` with `tui` subcommand
  * CLI arguments: deck paths, --p1/--p2 agent types, --seed
  * ZeroController: always chooses first meaningful action (filters out mana tap)
  * Successfully runs games to completion with zero and random controllers
  * Loads decks and card database from cardsfolder
  * Displays game results (winner, turns, life totals)
  * End-to-end test suite (4 tests) verifying:
    - Game completion with deterministic seeds
    - Reproducible results with same seed
    - Game state validation
    - Random vs Random successfully deals damage and wins by player death

- ✅ **Keyword ability support (K: lines)**
  * Keyword enum with 15+ evergreen keywords (Flying, Vigilance, etc.)
  * Protection variants (from red, blue, black, white, green)
  * Keywords with parameters: Madness, Flashback, Enchant
  * Parser handles both simple and parameterized keywords
  * Card struct has `keywords` field with helper methods (has_keyword, has_flying)
  * Comprehensive test suite (10 tests) for keyword loading
  * Tested on 10 diverse cards from cardsfolder

- ✅ **Basic ability parser for spell effects**
  * Parses "NumDmg$" from DealDamage abilities (A:SP$ DealDamage)
  * Auto-targets opponents for effects with no target
  * RandomController successfully casts Lightning Bolts and deals damage!

- ✅ **Creature combat system (COMPLETE!)**
  * Combat state tracking (attackers, blockers, damage assignment)
  * Declare attackers step with attacker selection
  * Declare blockers step with blocker assignment
  * Combat damage calculation (blocked and unblocked)
  * Automatic creature death when damage >= toughness
  * Full integration with game loop
  * Comprehensive tests (4 new combat tests)

- ✅ **Discard phase in cleanup step**
  * Players discard down to maximum hand size (default 7)
  * Added max_hand_size field to Player struct
  * Cleanup step calls controller.choose_cards_to_discard()
  * All controllers implement discard logic (Zero, Random, Scripted, combat_demo)
  * Active player discards first, then non-active players
  * Cards move from hand to graveyard
  * Comprehensive test verifies discard functionality
  * Fixes issue where players accumulated 39+ cards in hand

- ✅ **Performance benchmarking with Criterion.rs**
  * Comprehensive benchmark infrastructure with allocation tracking
  * **Fresh mode**: Allocate new game each iteration (~143µs per game)
  * **Snapshot mode**: Clone initial state each iteration (~131µs per game, 8% faster!)
  * **Rewind mode**: Placeholder for future undo() implementation
  * GameMetrics tracking:
    - Execution time, turns, actions
    - games/sec (~7,000), actions/sec (~400,000), turns/sec (~430,000)
    - Allocation tracking: bytes allocated/deallocated/net per game
    - ~374KB allocated, ~292KB deallocated, ~82KB net per game
    - ~4.2KB allocated per turn
  * Uses stats_alloc for allocation tracking
  * Profiling support:
    - Dedicated `profile` binary with CLI argument for iteration count
    - `make profile` - CPU time profiling with cargo-flamegraph (1000 games)
    - `make heapprofile` - Allocation profiling with cargo-heaptrack (100 games)
    - Flexible iteration count: `cargo run --bin profile -- 50`
    - Cleaner profiles without Criterion overhead
  * Run with `cargo bench` or `make bench`
  * Disabled RandomController stdout output for quiet benchmarking

- ✅ **Async card loading infrastructure**
  * AsyncCardDatabase with tokio-based parallel I/O
  * Two loading modes:
    - **Default (sync)**: Traditional synchronous loading for compatibility
    - **Eager async (--eager-load-cards)**: Parallel async loading of all cards
  * Lazy on-demand loading: Load only cards needed for decks
  * Card name to path conversion: "Lightning Bolt" → "cardsfolder/l/lightning_bolt.txt"
  * Thread-safe caching with Arc<RwLock<HashMap>>
  * Timing metrics:
    - Eager load: ~364ms for 31,438 cards (parallel)
    - Deck load: ~0.08ms for deck-specific cards (after eager load)
  * Comprehensive tests for async loading and caching
  * CLI flag: `--eager-load-cards` for parallel loading mode

## ✅ Phase 2 Complete: Game Loop Implementation

### Completed Features:
- ✅ **Complete game loop with turn phases and priority system**
  * All 11 turn steps (Untap, Upkeep, Draw, Main1, Combat steps, Main2, End, Cleanup)
  * Priority passing system with round-robin
  * Win condition checking (life <= 0, empty library)
  * Turn-based state management
  * Safety limits to prevent infinite loops
  * Fixed RandomController infinite loop bug

- ✅ **AI vs AI example game**
  * Demonstrates complete game from start to finish
  * Uses RandomController for both players
  * Loads decks from cardsfolder
  * Games complete successfully with win conditions

### Previously Completed (Phase 1):
1. Core entity system with unified EntityID generator
2. Card, Player, Mana, and GameState types
3. Game zones and turn structure
4. Game actions (play land, cast spells, deal damage, tap for mana)
5. Lightning Bolt MVP demo - fully playable!
6. Phantom types for EntityId<T> - type-safe IDs
7. Strong string types (CardName, PlayerName, Subtype)
8. Comprehensive CounterType enum (220+ counter types)
9. Proper mana payment system
10. Card effect system (6 basic effects)
11. Integrated undo log system
12. Development Makefile (build, test, validate)
13. Controller-driven game architecture
    - PlayerController trait for polymorphic controllers
    - GameStateView with zero-copy access
    - ScriptedController for testing
14. Card loading and database system
    - CardLoader parses .txt files from cardsfolder
    - CardDatabase indexes all cards (case-insensitive lookup)
    - DeckLoader parses .dck deck files
    - GameInitializer creates games from two decks
15. RandomController AI - baseline AI for testing
16. ZeroController - always chooses first meaningful action (for testing)
17. Main binary with TUI subcommand
    - Command-line interface with clap
    - Zero and random controller support
    - Seed-based deterministic games

---

### Next Priorities:

- [ ] **Enhanced TUI features**
  * Add random controller support (--p1=random)
  * Add interactive TUI controller (--p1=tui) for human play
  * Display game state during play (life, hand, battlefield)
  * Show available actions to player
  * Better formatting and colors in output

- [ ] **Enhanced creature support**
  * Summoning sickness tracking (needs turn-entered-battlefield tracking)
  * Vigilance keyword (attacking without tapping)
  * Flying/reach for combat restrictions
  * Multiple blockers support
  * Damage assignment order

- [ ] **More card types**
  - [ ] Creature cards (currently partially supported)
  - [ ] Enchantment cards
  - [ ] Artifact cards
  - [ ] Planeswalker cards (lower priority)

- [ ] **Ability system expansion** (see CARD_SCRIPT_SPEC.md for full DSL documentation)
  - [x] Keywords (K:) - Flying, First Strike, Protection, Madness, Flashback, Enchant, etc.
  - [x] Basic DealDamage parsing (A:SP$ DealDamage with NumDmg$)
  - [ ] More spell effects (A:SP$) - Draw, Counter, Destroy, Pump, GainLife
  - [ ] Activated abilities (A:AB$ with Cost$) - tap abilities, mana abilities
  - [ ] Triggered abilities (T:) - ETB, phase triggers, combat triggers
  - [ ] Static abilities (S:) - continuous effects
  - [ ] SVar resolution (DB$ sub-abilities)

- [ ] **Complex targeting**
  - [ ] Target validation (legal targets)
  - [ ] Target selection by controllers
  - [ ] "Any target" vs creature-only vs player-only

---

## 📋 Phase 4: Performance & Tree Search (LATER)

### Performance:
- [ ] Criterion benchmarks for key operations
- [ ] Undo/redo performance testing
- [ ] Board state evaluation speed
- [ ] Tree search optimization

### AI & Search:
- [ ] Implement undo() to rewind game state
- [ ] Tree search using undo log
- [ ] Basic board state evaluator
- [ ] MCTS or minimax search implementation
- [ ] Measure boardstates-per-second

---

## 📝 Phase 5: Advanced Features (FUTURE)

### More Game Mechanics:
- [ ] Stack interaction (responding to spells)
- [ ] Counterspells and instant-speed interaction
- [ ] Card draw triggers and replacement effects
- [ ] Discard mechanics
- [ ] Graveyard interactions
- [ ] Token creation
- [ ] +1/+1 and -1/-1 counters on creatures
- [ ] Equipment and Auras

### Serialization & Testing:
- [ ] GameState text file format (.pzl files)
- [ ] Load game states from files for testing
- [ ] Puzzle mode for testing specific scenarios
- [ ] Replay recorded games

### Advanced Performance:
- [ ] Fast binary game snapshots (rkyv)
- [ ] Parallel game search
- [ ] SIMD optimizations where applicable

---

## 🐛 Known Issues

None currently - all tests passing!

**Recently Fixed:**
- ✅ AI vs AI infinite loop (RandomController kept tapping lands indefinitely)

---

## 📊 Progress Summary

**Architecture:** ✅ Complete
**Core Game Engine:** ✅ Complete
**Game Loop:** ✅ Complete (including cleanup/discard phase)
**Combat:** ✅ Complete
**Keywords:** ✅ Complete (K: lines)
**Spell Effects:** 🚧 In Progress (DealDamage done, more needed)
**Triggered/Static Abilities:** 📋 Planned (T:, S: lines)
**Performance/Benchmarking:** ✅ Complete (Criterion.rs benchmarks)
**Tree Search:** 📋 Planned (needs undo() implementation)
**Advanced Features:** 📝 Future

**Total Tests:** 95 passing (80 unit + 10 card loading + 5 e2e)
**Test Coverage:** Good (core functionality + keyword parsing + discard phase + async loading)
**Performance:**
  - Fresh mode: ~143µs per game (~7,000 games/sec)
  - Snapshot mode: ~131µs per game (~7,600 games/sec, 8% faster)
  - Memory: ~82KB net allocation per game (~4.2KB per turn)
