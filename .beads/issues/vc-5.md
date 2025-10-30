---
title: Complete land evaluation in GameStateEvaluator
status: open
priority: 3
issue_type: task
created_at: "2025-10-30T05:28:25Z"
updated_at: "2025-10-30T05:28:25Z"
---

# Description

Implement detailed land evaluation heuristics.

Reference: GameStateEvaluator.java:240-285

Currently uses a simple heuristic. Should implement:
- +100 per mana the land can produce
- +3 per color of mana produced
- +25 for manlands (activated abilities without tap cost)
- +10 for sac abilities (not repeatable)
- +50 for repeatable utility abilities (tap cost)
- +6 per static ability

Requires parsing mana abilities to determine:
- How much mana produced
- What colors produced
- Cost to activate
