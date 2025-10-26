---
title: Mana pool calculation optimization
status: open
priority: 4
issue_type: chore
created_at: "2025-10-26T21:06:34Z"
updated_at: "2025-10-26T21:06:34Z"
---

# Description

Review ManaEngine operations for unnecessary cloning of mana costs.
Seen in game_loop.rs:106,277 (mana_cost.clone()).
