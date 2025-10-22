# MTG Forge Rust - TODO

## Current Status

**Tests:** 133 passing ✅ (115 lib + 10 card_loading + 3 determinism + 5 tui) | **Validation:** `make validate` passes all checks ✅

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
- ✅ **Summoning sickness tracking** - Creatures can't attack the turn they enter battlefield
  - Added `turn_entered_battlefield` field to Card struct
  - Set when permanents enter battlefield (via play_land or resolve_spell)
  - Validated in declare_attacker to prevent illegal attacks
  - Haste keyword bypasses summoning sickness
  - Full test coverage (3 new tests)
- ✅ **Vigilance keyword** - Creatures with vigilance don't tap when attacking
  - Modified declare_attacker to check for Vigilance keyword before tapping
  - Creatures without vigilance tap normally when attacking
  - Full test coverage (2 new tests)
- ✅ **Flying/Reach combat restrictions** - Flying creatures can only be blocked by flying/reach creatures
  - Creatures with Flying keyword can only be blocked by creatures with Flying or Reach
  - Creatures with Reach can block Flying creatures
  - Validation in declare_blocker enforces MTG rule 702.9
  - Full test coverage (5 new tests: flying vs flying, flying vs reach, flying vs ground, ground vs any, flying+reach)
- ✅ **First Strike and Double Strike combat damage** - Two combat damage steps when first strike is present
  - Implements MTG Rules 510.4: Two combat damage steps if any creature has first strike or double strike
  - First strike creatures deal damage before creatures without first strike
  - Double strike creatures deal damage in both first strike and normal damage steps
  - Dead creatures from first strike don't deal damage in normal step
  - Helper methods: `has_first_strike()`, `has_double_strike()`, `has_normal_strike()`
  - Full test coverage (4 new tests: first strike kills first, double strike hits twice, double vs first, normal vs first)
- ✅ **Draw spell effects** - Cards that draw cards now work (e.g., Ancestral Recall, Divination)
  - Parser recognizes `SP$ Draw | NumCards$ X` abilities from card definitions
  - Execution implemented via `Effect::DrawCards` with automatic player targeting
  - Placeholder player ID 0 replaced with card controller during resolution
  - Full test coverage (3 tests: parsing, execution, end-to-end spell resolution)
- ✅ **Destroy spell effects** - Cards that destroy permanents now work (e.g., Terror, Murder)
  - Parser recognizes `SP$ Destroy | ValidTgts$ Creature` abilities from card definitions
  - Execution implemented via `Effect::DestroyPermanent` with automatic targeting
  - Placeholder card ID 0 replaced with opponent's creature during resolution
  - Full test coverage (2 tests: parsing Terror, end-to-end spell resolution)
- ✅ **GainLife spell effects** - Cards that gain life now work (e.g., Angel's Mercy)
  - Parser recognizes `SP$ GainLife | LifeAmount$ X` abilities from card definitions
  - Execution implemented via `Effect::GainLife` with automatic player targeting
  - Placeholder player ID 0 replaced with card controller during resolution
  - Full test coverage (2 tests: parsing Angel's Mercy, end-to-end spell resolution)
- ✅ TUI support: `mtg tui` command with --p1/--p2 agent types (zero/random), --seed for deterministic games
- ✅ Keyword abilities (K: lines): 15+ keywords including Flying, Vigilance, Protection, Madness, Flashback
- ✅ Spell effects: DealDamage (Lightning Bolt), Draw (Ancestral Recall), Destroy (Terror), GainLife (Angel's Mercy)
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
  * ✅ Add random controller support (--p1=random)
  * Add interactive TUI controller (--p1=tui) for human play
  * Display game state during play (life, hand, battlefield)
  * Show available actions to player
  * Better formatting and colors in output

- [ ] **Enhanced creature support**
  * ✅ Summoning sickness tracking
  * ✅ Vigilance keyword
  * ✅ Flying/reach for combat restrictions
  * ✅ Multiple blockers support
  * ✅ Damage assignment order
  * ✅ First strike / Double strike combat damage

- [ ] **More card types**
  - [ ] Creature cards (currently partially supported)
  - [ ] Enchantment cards
  - [ ] Artifact cards
  - [ ] Planeswalker cards (lower priority)

- [ ] **Ability system expansion** (see CARD_SCRIPT_SPEC.md for full DSL documentation)
  - [x] Keywords (K:) - Flying, First Strike, Protection, Madness, Flashback, Enchant, etc.
  - [x] Basic DealDamage parsing (A:SP$ DealDamage with NumDmg$)
  - [x] Draw spell effects (A:SP$ Draw with NumCards$)
  - [x] Destroy spell effects (A:SP$ Destroy with ValidTgts$)
  - [x] GainLife spell effects (A:SP$ GainLife with LifeAmount$)
  - [ ] More spell effects (A:SP$) - Counter, Pump
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
- [ ] **Optimization work** - See [OPTIMIZATION.md](OPTIMIZATION.md) for best practices and backlog
  - Zero-copy patterns (avoid clone/collect where possible)
  - Arena allocation for per-turn temporaries
  - Object pools for reusable objects
  - Heap profiling to identify allocation hotspots
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
**Phase 3 (Gameplay):** 🚧 In Progress - Combat ✅, Keywords ✅, Basic Spells ✅, ManaEngine ✅, Logging ✅, Summoning Sickness ✅, Vigilance ✅, Benchmarking ✅, Async Loading ✅
**Phase 4 (Performance/AI):** 📋 Planned
**Phase 5 (Advanced Features):** 📝 Future

**Tests:** 133 passing | **Performance:** ~7,000 games/sec, 82KB/game | **Cards:** 31k+ supported
