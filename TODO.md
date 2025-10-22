# MTG Forge Rust - TODO

## Current Status

**Tests:** 112 passing ✅ (94 lib + 10 card_loading + 3 determinism + 5 tui) | **Validation:** `make validate` passes all checks ✅

---

## ✅ Completed Work

### Phase 3: Gameplay Expansion (In Progress)
- ✅ **Controller architecture refactoring** - Aligned with Java Forge PlayerController.java
  - Unified spell ability selection: `choose_spell_ability_to_play()` handles lands, spells, and abilities
  - Correct mana timing: Mana tapped during step 6 of 8-step casting process (AFTER spell on stack)
  - 8-step spell casting: Implements MTG Rules 601.2 (propose, targets, cost, mana, pay, cast)
  - Updated RandomController and ZeroController to new interface
  - Removed incorrect mana-tapping-before-casting behavior
  - See REFACTORING_STATUS.md for full details
- ✅ **Mana cost payability computation** - Per-player ManaEngine for efficient mana checking
  - Partitions lands into simple (basic lands) vs complex (dual/choice lands)
  - Caches simple sources as WUBRGC counters for O(1) queries
  - `can_pay()` method correctly handles generic vs colorless mana
  - Integrated into spell castability checking (replaces simple CMC check)
  - Complex sources stubbed with todo! for future implementation
- ✅ **Enhanced game event logging** - Improved visibility at Normal verbosity level (--verbosity=2)
  - Land plays: "Player plays Forest"
  - Spell casting: "Player casts Grizzly Bears (putting on stack)"
  - Spell resolution: "Grizzly Bears resolves, enters the battlefield as a 2/2 creature"
  - Combat attackers: "Player declares Grizzly Bears (2/2) as attacker"
  - Combat damage: "Grizzly Bears deals 2 damage to Player" and "Combat: X ↔ Y"
- ✅ TUI support: `mtg tui` command with --p1/--p2 agent types, --seed for deterministic games
- ✅ Keyword abilities (K: lines): 15+ keywords including Flying, Vigilance, Protection, Madness, Flashback
- ✅ Basic spell effects: DealDamage parsing, Lightning Bolt works
- ✅ Creature combat: attackers, blockers, damage calculation, creature death
- ✅ Cleanup/discard phase: players discard to max hand size
- ✅ Benchmarking: Criterion.rs infrastructure (~7,000 games/sec, 82KB/game allocation)
- ✅ Async card loading: jwalk streaming discovery, deck-only or --load-all-cards modes

### Phase 2: Game Loop
- ✅ Complete turn system: all 11 steps, priority passing, win conditions
- ✅ AI vs AI demo with RandomController

### Phase 1: Core Architecture
- ✅ Entity system, game state, zones, actions, mana payment
- ✅ Type-safe IDs, strong types, undo logging
- ✅ Controller architecture: PlayerController trait, GameStateView, Random/Zero/Scripted controllers
- ✅ **Two-layer controller architecture (v2)**: Specific callbacks (PlayerController) + generic choices (DecisionMaker)
  - RandomControllerV2 and ZeroControllerV2 with zero-copy patterns (SmallVec, slices)
  - Specific methods: choose_land_to_play, choose_spell_to_cast, choose_attackers, choose_blockers, etc.
  - Documentation in CONTROLLER_DESIGN.md
  - Note: Game loop still uses v1 interface, v2 migration pending
- ✅ Card/deck loading from cardsfolder .txt and .dck files

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

## ⚡ Performance Optimization Backlog

### Recently Completed:
- ✅ **Eliminated player ID collect() calls** in game loop hot paths
  - Replaced 10+ `collect()` calls with direct iterator access
  - Used `.find()` instead of collecting Vec then indexing
  - **Result:** Fresh mode 1.2-2.2% faster, Snapshot mode 11-13% faster
  - Files: game_loop.rs, main.rs, benches/game_benchmark.rs

### High Priority - Allocation Hot Spots:
- [ ] **CardDatabase::get_card() returns references** (Major)
  - Currently clones CardDefinition on every access (line 52, database_async.rs)
  - Heaptrack shows this as top allocation site
  - Requires adding lifetime parameters to return `&CardDefinition`
  - Would eliminate ~50% of Card struct clones

- [ ] **Eliminate GameStateView clones** (Medium)
  - Created on every controller decision
  - Consider borrowing instead of cloning where possible

- [ ] **String allocation optimization** (Medium)
  - Card names, player names cloned frequently
  - Consider using Arc<str> or &'static str where appropriate
  - Log messages allocate heavily - consider conditional compilation

### Lower Priority - Legitimate Uses:
- These collect() calls are necessary for borrow checker but documented for awareness:
  - reset_turn_state (line 298): collect player IDs before mutating
  - untap_step (line 357): collect card IDs before mutating
  - get_next_player (state.rs:247): could optimize for 2-player case

### Measurement Notes:
- Benchmark before: Fresh ~162µs/game, Snapshot ~166µs/game
- Benchmark after: Fresh ~159µs/game, Snapshot ~147µs/game
- Heaptrack showed ~4GB allocations per 10k games before optimizations

---

## 📊 Progress Summary

**Phase 1 (Core Architecture):** ✅ Complete
**Phase 2 (Game Loop):** ✅ Complete
**Phase 3 (Gameplay):** 🚧 In Progress - Combat ✅, Keywords ✅, Basic Spells ✅, ManaEngine ✅, Logging ✅, Benchmarking ✅, Async Loading ✅
**Phase 4 (Performance/AI):** 📋 Planned
**Phase 5 (Advanced Features):** 📝 Future

**Tests:** 112 passing | **Performance:** ~7,000 games/sec, 82KB/game | **Cards:** 31k+ supported
