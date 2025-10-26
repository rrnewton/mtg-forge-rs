---
title: Port evalManaBase() to GameStateEvaluator
status: open
priority: 3
issue_type: task
created_at: "2025-10-26T21:06:34Z"
updated_at: "2025-10-26T21:06:34Z"
---

# Description

Port the evalManaBase() function from Java to Rust GameStateEvaluator.

Reference: GameStateEvaluator.java:176-216

The function evaluates mana base quality by:
- Counting available mana sources by color
- Comparing against deck's color requirements (maxPips)
- Comparing total mana against deck's max CMC
- Value: +100 per color pip up to deck needs
- Value: +100 per total mana source up to max CMC
- Value: +5 per excess mana beyond deck needs

Requires:
- Access to player's lands and mana abilities
- Deck statistics (max pips by color, max CMC)
- Mana ability parsing to determine colors produced
