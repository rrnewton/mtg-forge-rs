---
title: Migrate game loop from v1 to v2 controller interface
status: open
priority: 3
issue_type: feature
created_at: "2025-10-26T21:06:34Z"
updated_at: "2025-10-26T21:06:34Z"
---

# Description

Complete migration to v2 controller architecture:
- Game loop currently uses v1 interface
- v2 provides better zero-copy patterns (SmallVec, slices)
- v2 has specific callbacks (choose_land_to_play, choose_spell_to_cast, etc.)
- See CONTROLLER_DESIGN.md for architecture details
