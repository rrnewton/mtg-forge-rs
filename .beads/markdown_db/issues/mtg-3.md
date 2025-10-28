---
title: 'MTG feature completeness: keywords, abilities, effects'
status: open
priority: 1
issue_type: epic
labels:
  - tracking
created_at: "2025-10-26T21:06:34Z"
updated_at: "2025-10-26T21:06:34Z"
---

# Description

Track implementation of Magic: The Gathering game mechanics.

**Card types:**
- Creature cards: ✅ Complete (combat, summoning sickness, keywords)
- Enchantment cards: ✅ Basic support (cast, enters battlefield)
- Artifact cards: ✅ Basic support (cast, enters battlefield)
- mtg-16: Aura enchantments (need enchant targeting)
- mtg-17: Equipment artifacts (need equip abilities)
- mtg-18: Planeswalker cards (lower priority)

**Ability system (see ai_docs/CARD_SCRIPT_SPEC.md):**
- Keywords (K:): ✅ 16+ keywords implemented
- Spell effects (A:SP$): ✅ DealDamage, Draw, Destroy, GainLife, Pump, Tap, Untap, Mill, Counter
- Activated abilities (A:AB$): ✅ Basic execution with tap/mana costs
- Mana abilities: ✅ AB$ Mana production (basic lands only)
- mtg-108: **Complex mana source handling** (dual lands, City of Brass, etc.) - blocks Old School decks
- mtg-19: Advanced activated abilities (complex costs, stack interaction, player choice for "Any"/"Combo" mana)
- Triggered abilities (T:): ✅ ETB triggers with multiple effect types
- mtg-20: Static abilities (S:) - continuous effects
- mtg-21: SVar resolution (DB$ sub-abilities)

**Targeting:**
- mtg-22: Target validation (legal targets)
- mtg-23: Target selection by controllers
- mtg-24: "Any target" vs creature-only vs player-only

**AI (see mtg-77 for detailed tracking):**
- ✅ HeuristicController with creature evaluation
- ✅ Combat decisions (attack/block)
- mtg-77: Complete heuristic AI port from Java Forge

**Related Tracking Issues:**
- mtg-108: Complex mana source handling (phased implementation)
- mtg-77: Heuristic AI completeness
