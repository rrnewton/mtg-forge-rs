---
title: Heuristic AI completeness tracking
status: open
priority: 1
issue_type: epic
created_at: "2025-10-26T21:06:34Z"
updated_at: "2025-10-26T21:06:51Z"
---

# Description

Track completion of heuristic AI port from Java Forge to Rust.

## Current Status

**What's Implemented in HeuristicController:**
- ✅ Creature evaluation (comprehensive, faithful port)
- ✅ Attack decisions with aggression levels (basic - needs improvement)
- ✅ Block decisions with value trading
- ✅ Basic targeting (best creature)
- ✅ Basic spell selection (creatures first)
- ✅ GameStateEvaluator (basic holistic board evaluation)
- ✅ Opponent life access (bd-4 completed)

**What's Missing:**

### High Priority (Core AI Strength):

1. **Attack logic improvements (mtg-85)**
   - Current: Only evaluates attacker stats in isolation
   - Missing: Board state evaluation, combat math, blockability checks
   - Reference: Java's SpellAbilityFactors class in AiAttackController.java:1350-1562
   - Impact: 2/2 vanilla creatures never attack even with no blockers

2. **GameStateEvaluator improvements:**
   - mtg-78: Port evalManaBase() - mana base quality scoring
   - mtg-79: Track summon sickness properly
   - mtg-81: Complete land evaluation (detailed heuristics)

3. **Combat outcome prediction**
   - Simulate combat before making decisions
   - Critical for knowing if attacks will be lethal
   - Reference: GameStateEvaluator.java:40-67, 91-100

### Medium Priority:

4. **Spell evaluation** - Beyond creatures
   - Removal spell targeting (ComputerUtilCard)
   - Card draw value assessment
   - Pump spells, combat tricks

5. **Mana tapping order** - ComputerUtilMana
   - Leave up correct colors for instant responses
   - Optimize painland/fetchland usage

### Lower Priority:

6. **Damage assignment order** - Kill blockers efficiently
7. **Bluffing/deception** - Hold information when advantageous
8. mtg-80: Improve enchantment evaluation

## Completed Work

- ✅ Basic GameStateEvaluator with hand, life, and battlefield evaluation
- ✅ Creature evaluation (faithful port from Java)
- ✅ Basic land evaluation
- ✅ Score type with summon sickness tracking
- ✅ Opponent life access (bd-4) - GameStateView now provides player_life(), opponents(), and opponent_life() methods
- ✅ Activated ability targeting (mtg-70) - Royal Assassin can now target and destroy tapped creatures

