---
title: Stop-and-go game snapshots for interactive exploration
status: open
priority: 3
issue_type: feature
created_at: "2025-10-27T00:03:50Z"
updated_at: "2025-10-27T00:03:50Z"
---

# Description

Implement game snapshot/resume feature:
- Add --stop-every=[p1|p2|both]:choice:<NUM> flag to stop at choice points
- Add --snapshot-output flag (default: game.snapshot)
- Add --start-from flag to load and resume from snapshot
- Serialize GameState to JSON when stopping
- Resume from exact game state including re-echoing prompts
- Enables interactive game tree exploration
- Test with royal_assassin.dck vs royal_assassin.dck

This stresses our serialization support and makes the game explorable like a debugger.
