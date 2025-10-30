---
title: Improve error messages in card loader
status: closed
priority: 0
issue_type: task
created_at: "2025-10-30T05:28:25Z"
updated_at: "2025-10-30T05:28:25Z"
closed_at: "2025-10-24T10:15:52Z"
---

# Description

When cards fail to load from the cardsfolder, the error messages should be more descriptive.
Current behavior: Generic "failed to load card" message
Desired behavior: Show which field failed to parse, line number, and suggested fix
This will save debugging time when adding new cards.
Add profile command to CLI
Add a `profile` subcommand that runs heaptrack profiling automatically.
Should:
- Run a configurable number of games
- Generate heaptrack output
- Automatically analyze results
- Print top allocation sites
Example: `mtg profile --games 1000 --seed 42`
Document the mana engine API
The ManaEngine is complex and needs better documentation.
Add:
- Module-level docs explaining the architecture
- Examples of common use cases
- Diagram showing interaction with GameState
- Performance characteristics and optimization notes
Target audience: contributors implementing new card effects.
