---
title: Improve enchantment evaluation in GameStateEvaluator
status: open
priority: 4
issue_type: task
created_at: "2025-10-30T05:28:25Z"
updated_at: "2025-10-30T05:28:25Z"
---

# Description

Properly evaluate enchantments based on what they're enchanting.

Reference: GameStateEvaluator.java:224-228

Currently enchantments have 0 value. Java comment says:
"Should only provide value based on what it's enchanting. Else the AI would think that casting a Lifelink enchantment on something that already has lifelink is a net win."

This requires:
- Tracking what permanents enchantments are attached to
- Evaluating the enchantment's effect on the enchanted permanent
- Not double-counting abilities already present
