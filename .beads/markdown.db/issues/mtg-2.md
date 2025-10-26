---
title: Optimization and performance tracking
status: open
priority: 1
issue_type: epic
created_at: "2025-10-26T21:06:34Z"
updated_at: "2025-10-26T21:06:34Z"
---

# Description

Track performance optimization work for MTG Forge Rust.

**Current performance as of commit#162(387498cecf):**
- Games/sec: ~4,694 (seed 42, fresh mode)
- Actions/sec: ~338,000
- Turns/sec: ~413,000
- Actions/turn: 0.82
- Avg allocations/turn: 25,890 bytes
- Avg duration/game: 213.03µs

**High priority optimization issues:**
- mtg-6: Logging allocations (#1 hotspot - 70%+ of allocations)
- mtg-7: CardDatabase.get_card() should return references
- mtg-8: Eliminate GameStateView clones
- mtg-9: String allocation optimization

**Medium priority:**
- mtg-10: Vec reallocations in game loop
- mtg-11: Zone transfer operations optimization
- mtg-12: Mana pool calculation optimization

**Future considerations:**
- mtg-13: Arena allocation for per-turn temporaries
- mtg-14: Object pools for reusable objects
- mtg-15: Compile-time feature flags for profiling modes

See OPTIMIZATION.md for detailed analysis and profiling methodology.
