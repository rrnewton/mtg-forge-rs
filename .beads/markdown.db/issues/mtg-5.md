---
title: 'Cross-cutting codebase issues: APIs, testing, architecture'
status: open
priority: 1
issue_type: epic
created_at: "2025-10-26T21:06:34Z"
updated_at: "2025-10-26T21:06:34Z"
---

# Description

Track architectural improvements, API design, and testing infrastructure.

**Controller architecture:**
- Current: Two-layer controller architecture (v2) documented in CONTROLLER_DESIGN.md
  - PlayerController trait with specific callbacks
  - DecisionMaker with generic choices
  - RandomControllerV2 and ZeroControllerV2 with zero-copy patterns (SmallVec, slices)
- mtg-40: Migrate game loop from v1 to v2 controller interface
- mtg-41: Controller API consistency and documentation

**Testing infrastructure:**
- Current: 188 passing tests (165 lib + 10 card_loading + 4 determinism + 7 tui + 2 undo)
- mtg-42: Improve test coverage for edge cases
- mtg-43: Integration test suite expansion
- mtg-44: Determinism testing for more complex scenarios
- mtg-45: Property-based testing with proptest

**Performance & Tree Search (Phase 4):**
- mtg-46: Undo/redo performance testing
- mtg-47: Board state evaluation function
- mtg-48: Tree search using undo log
- mtg-49: MCTS or minimax search implementation
- mtg-50: Measure boardstates-per-second

**Serialization:**
- mtg-51: Fast binary game snapshots (rkyv)
- mtg-52: Parallel game search capabilities
- mtg-53: SIMD optimizations where applicable
