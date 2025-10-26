---
title: Overall MTG Forge Rust development tracking
status: open
priority: 0
issue_type: epic
created_at: "2025-10-26T21:06:34Z"
updated_at: "2025-10-26T21:06:34Z"
---

# Description

This is the main tracking issue for MTG Forge Rust development.

**Major tracking issues (priority 1):**
- mtg-2: Optimization and performance tracking
- mtg-3: MTG feature completeness (keywords, abilities, effects)
- mtg-4: Gameplay features (TUI, human play, controls)
- mtg-5: Cross-cutting codebase issues (APIs, testing, architecture)
- mtg-77: Heuristic AI completeness tracking

**Current status as of 2025-10-26_#333(32cd76fe):**
- Tests: 312 passing (nextest, all categories)
- Examples: 14/14 passing
- Performance: ~3,842 games/sec (fresh mode), 16.56 actions/turn
- Performance: ~9,177 games/sec (snapshot mode), ~332k rewinds/sec (rewind mode)
- Cards: 31k+ supported from cardsfolder

**Conventions:**
- Tracking issues (priority 1) reference granular issues
- Granular issues have priority 3-4 unless critical bugs (priority 2)
- Human-created issues have priority 0
- Reference issues in code: // TODO(mtg-N): description
- Transient info includes timestamp: commit#N(hash)

---
**Checked up-to-date as of 2025-10-26_#333(32cd76fe)**
- Updated tracking issue references (corrected mtg-63→mtg-2, etc.)
- Updated test count: 188 → 312 tests
- Updated example count: 13 → 14 examples
- Updated performance metrics with all three benchmark modes
