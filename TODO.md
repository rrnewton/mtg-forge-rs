# MTG Forge Rust - TODO

## Current Status

**Tests:** 174 passing ✅ (151 lib + 10 card_loading + 4 determinism + 7 tui + 2 undo) | **Validation:** `make validate` passes all checks ✅

---

## ✅ Completed Work

### Phase 3: Gameplay Expansion (In Progress)
- ✅ **Controller architecture refactoring** - Aligned with Java Forge PlayerController.java
- ✅ **Mana cost payability computation** - Per-player ManaEngine for efficient mana checking
- ✅ **Enhanced game event logging** - Improved visibility at Normal verbosity level (--verbosity=2)
- ✅ **Random choice logging** - Logs each random decision made by RandomController
- ✅ **Undo/replay system fixes** - Turn counter and step progression properly tracked and undone
- ✅ **Undo/replay discard bug fix** - Cleanup step now properly logs card discards for undo
- ✅ **Summoning sickness tracking** - Creatures can't attack the turn they enter battlefield
- ✅ **Vigilance keyword** - Creatures with vigilance don't tap when attacking
- ✅ **Flying/Reach combat restrictions** - Flying creatures can only be blocked by flying/reach creatures
- ✅ **First Strike and Double Strike combat damage** - Two combat damage steps when first strike is present
- ✅ **Draw spell effects** - Cards that draw cards now work (e.g., Ancestral Recall, Divination)
- ✅ **Destroy spell effects** - Cards that destroy permanents now work (e.g., Terror, Murder)
- ✅ **GainLife spell effects** - Cards that gain life now work (e.g., Angel's Mercy)
- ✅ **Pump spell effects** - Cards that temporarily boost creature stats now work (e.g., Giant Growth)
- ✅ **Tap/Untap spell effects** - Cards that tap or untap permanents now work
- ✅ **Mill spell effects** - Cards that mill cards from library to graveyard now work (e.g., Thought Scour, Mind Sculpt)
- ✅ **ETB (Enters the Battlefield) triggers** - Triggered abilities when permanents enter battlefield (e.g., Elvish Visionary, Flametongue Kavu)
- ✅ **Trample keyword** - Excess combat damage tramples over to defending player
- ✅ **Lifelink keyword** - Creatures with lifelink gain life equal to damage dealt
- ✅ **Deathtouch keyword** - Any damage from deathtouch source destroys creature
- ✅ **Menace keyword** - Creatures with menace can't be blocked except by two or more creatures
- ✅ **Hexproof keyword** - Creatures with hexproof can't be targeted by opponent's spells or abilities
- ✅ **Indestructible keyword** - Permanents with indestructible can't be destroyed
- ✅ **Shroud keyword** - Permanents with shroud can't be targeted by any player
- ✅ **Defender keyword** - Creatures with defender can't attack
- ✅ TUI support: `mtg tui` command with --p1/--p2 agent types (zero/random), --seed for deterministic games
- ✅ Keyword abilities (K: lines): 16+ keywords including Flying, Vigilance, Trample, Lifelink, Deathtouch, Menace, Hexproof, Indestructible, Shroud, Defender, Protection, Madness, Flashback
- ✅ Spell effects: DealDamage (Lightning Bolt), Draw (Ancestral Recall), Destroy (Terror), GainLife (Angel's Mercy), Pump (Giant Growth), Tap/Untap, Mill (Thought Scour)
- ✅ Creature combat: attackers, blockers, damage calculation, creature death, Trample, Lifelink, Deathtouch
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

### Extra tasks added by human

 - [x] Elide random choices with one option only
   - ✅ RandomController no longer logs when there's no meaningful choice (single target, exact mana sources needed, etc.)
 - [ ]    


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
  * ✅ Trample keyword
  * ✅ Lifelink keyword
  * ✅ Deathtouch keyword
  * ✅ Flying/reach for combat restrictions
  * ✅ Multiple blockers support
  * ✅ Damage assignment order
  * ✅ First strike / Double strike combat damage
  * ✅ Menace keyword (requires at least 2 blockers)
  * ✅ Hexproof keyword (can't be targeted by opponents)

- [ ] **More card types**
  - [x] Creature cards (combat, summoning sickness, keywords)
  - [x] Enchantment cards (basic support - can cast and enter battlefield)
  - [x] Artifact cards (basic support - can cast and enter battlefield)
  - [ ] Aura enchantments (need enchant targeting)
  - [ ] Equipment artifacts (need equip abilities)
  - [ ] Planeswalker cards (lower priority)

- [ ] **Ability system expansion** (see CARD_SCRIPT_SPEC.md for full DSL documentation)
  - [x] Keywords (K:) - Flying, First Strike, Protection, Madness, Flashback, Enchant, etc.
  - [x] Basic DealDamage parsing (A:SP$ DealDamage with NumDmg$)
  - [x] Draw spell effects (A:SP$ Draw with NumCards$)
  - [x] Destroy spell effects (A:SP$ Destroy with ValidTgts$)
  - [x] GainLife spell effects (A:SP$ GainLife with LifeAmount$)
  - [x] Pump spell effects (A:SP$ Pump with NumAtt$/NumDef$)
  - [x] Tap/Untap spell effects (A:SP$ Tap, A:SP$ Untap)
  - [x] Mill spell effects (A:SP$ Mill with NumCards$)
  - [ ] More spell effects (A:SP$) - Counter
  - [ ] Activated abilities (A:AB$ with Cost$) - tap abilities, mana abilities
  - [x] Triggered abilities (T:) - ETB triggers with Draw and DealDamage effects (basic support)
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
**Phase 3 (Gameplay):** 🚧 In Progress - Combat ✅, Keywords (Flying/Vigilance/Trample/Lifelink/Deathtouch/Menace/Hexproof/Indestructible/Shroud/FirstStrike/DoubleStrike) ✅, Spell Effects (Damage/Draw/Destroy/GainLife/Pump/Tap/Untap) ✅, ManaEngine ✅, Logging ✅, Benchmarking ✅, Async Loading ✅
**Phase 4 (Performance/AI):** 📋 Planned
**Phase 5 (Advanced Features):** 📝 Future

**Tests:** 163 passing | **Performance:** ~7,000 games/sec, 82KB/game | **Cards:** 31k+ supported
