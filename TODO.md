# MTG Forge Rust - TODO

## Current Status

**Latest Commit:** c6028dd - Implement complete game loop with turn phases and priority

**Tests:** 67 passing ✅

---

## ✅ Phase 2 Complete: Game Loop Implementation

### Completed Features:
- ✅ **Complete game loop with turn phases and priority system**
  * All 11 turn steps (Untap, Upkeep, Draw, Main1, Combat steps, Main2, End, Cleanup)
  * Priority passing system with round-robin
  * Win condition checking (life <= 0, empty library)
  * Turn-based state management
  * Safety limits to prevent infinite loops

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

---

## 🚧 Phase 3: Gameplay Expansion (NEXT)

### Immediate Priorities:
- [ ] **Implement creature combat system**
  - [ ] Declare attackers step
  - [ ] Declare blockers step
  - [ ] Combat damage calculation and assignment
  - [ ] Death state-based actions for creatures

- [ ] **More card types**
  - [ ] Creature cards (currently partially supported)
  - [ ] Enchantment cards
  - [ ] Artifact cards
  - [ ] Planeswalker cards (lower priority)

- [ ] **Ability system**
  - [ ] Triggered abilities (on cast, on enter, on death, etc.)
  - [ ] Activated abilities (tap abilities, mana abilities)
  - [ ] Static abilities (continuous effects)
  - [ ] Ability parser for card scripts (A:, S:, T: lines)

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

---

## 📊 Progress Summary

**Architecture:** ✅ Complete
**Core Game Engine:** ✅ Complete
**Game Loop:** ✅ Complete
**Combat:** 🚧 Next priority
**Abilities:** 🚧 Next priority
**Performance/Search:** 📋 Planned
**Advanced Features:** 📝 Future

**Total Tests:** 67 passing
**Test Coverage:** Good (core functionality)
