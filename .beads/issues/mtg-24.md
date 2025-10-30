---
title: '"Any target" vs creature-only vs player-only targeting'
status: open
priority: 3
issue_type: feature
created_at: "2025-10-30T05:28:25Z"
updated_at: "2025-10-30T05:28:25Z"
---

# Description

Properly distinguish between target types:
- ValidTgts$Creature,Player (any target - can hit creatures or players)
- ValidTgts$Creature (creature-only)
- ValidTgts$Player (player-only)
- ValidTgts$Permanent (any permanent)

Update spell effects to respect these distinctions.
