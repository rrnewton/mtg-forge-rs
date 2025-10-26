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
- mtg-63: Optimization and performance tracking
- mtg-68: MTG feature completeness (keywords, abilities, effects)
- mtg-69: Gameplay features (TUI, human play, controls)
- mtg-70: Cross-cutting codebase issues (APIs, testing, architecture)
- vc-1: Heuristic AI completeness tracking

**Current status as of commit#162(387498cecf):**
- Tests: 188 passing (165 lib + 10 card_loading + 4 determinism + 7 tui + 2 undo)
- Examples: 13/13 passing
- Performance: ~4,694 games/sec, ~338k actions/sec
- Cards: 31k+ supported from cardsfolder

**Conventions:**
- Tracking issues (priority 1) reference granular issues
- Granular issues have priority 3-4 unless critical bugs (priority 2)
- Human-created issues have priority 0
- Reference issues in code: // TODO(mtg-N): description
- Transient info includes timestamp: commit#N(hash)
