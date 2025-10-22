# Controller Refactoring Plan

## Problems with Current Implementation

1. **❌ Split Land/Spell Playing**: We have separate `choose_land_to_play()` and `choose_spell_to_cast()` methods, but Java combines these in `chooseSpellAbilityToPlay()` because they're both playable abilities.

2. **❌ Wrong Mana Timing**: We're tapping for mana during priority rounds BEFORE casting spells. This violates MTG rules - mana should be tapped as **step 6 of 8** during spell casting, AFTER the spell is on the stack.

3. **❌ Incomplete Casting Sequence**: Our `cast_spell()` doesn't implement the full 8-step process from Rules 601.2.

## Java Forge Architecture (Source of Truth)

### Key Methods in PlayerController.java:
```java
public abstract List<SpellAbility> chooseSpellAbilityToPlay();
public abstract boolean playChosenSpellAbility(SpellAbility sa);
public abstract boolean chooseTargetsFor(SpellAbility currentAbility);
public abstract boolean payManaCost(ManaCost toPay, ...);
```

### SpellAbility Hierarchy:
- **SpellAbility** (abstract) - Any playable ability
  - **LandAbility** - Play land (resolves directly, no stack)
  - **Spell** - Cast spell (goes on stack, 8-step process)
  - **Ability** - Activated/Triggered abilities

### From Rules Section 5.3 - The 8 Steps of Casting:
1. **Propose**: Move card to stack
2. **Make Choices**: Announce modes, X values
3. **Choose Targets**: Select targets
4. **Divide Effects**: Divide damage/counters
5. **Determine Cost**: Calculate total cost
6. **Activate Mana Abilities**: ⚡ **NOW tap for mana**
7. **Pay Costs**: Pay the cost
8. **Spell Becomes Cast**: Trigger "whenever you cast" abilities

## Proposed Rust Architecture

### 1. SpellAbility Representation
```rust
#[derive(Debug, Clone)]
pub enum SpellAbility {
    /// Play a land from hand (resolves directly, no stack)
    PlayLand { card_id: CardId },

    /// Cast a spell (goes through 8-step process)
    CastSpell { card_id: CardId },

    /// Activate an ability on a permanent
    ActivateAbility {
        card_id: CardId,
        ability_index: usize
    },
}
```

### 2. New PlayerController Trait
```rust
pub trait PlayerController {
    fn player_id(&self) -> PlayerId;

    /// Choose which spell ability to play (combines lands, spells, abilities)
    /// Returns None to pass priority
    fn choose_spell_ability_to_play(
        &mut self,
        view: &GameStateView,
        available: &[SpellAbility],
    ) -> Option<SpellAbility>;

    /// Choose targets during step 3 of casting
    fn choose_targets(
        &mut self,
        view: &GameStateView,
        spell: CardId,
        valid_targets: &[CardId],
    ) -> SmallVec<[CardId; 4]>;

    /// Pay mana cost during step 6 by choosing sources to tap
    /// Returns cards to tap for mana in order
    fn choose_mana_sources_to_pay(
        &mut self,
        view: &GameStateView,
        cost: &ManaCost,
        available_sources: &[CardId],
    ) -> SmallVec<[CardId; 8]>;

    // Combat (unchanged)
    fn choose_attackers(...);
    fn choose_blockers(...);

    // Cleanup (unchanged)
    fn choose_cards_to_discard(...);

    // Notifications (unchanged)
    fn on_priority_passed(...);
    fn on_game_end(...);
}
```

### 3. Game Loop Changes

**Priority Round (game_loop.rs)**:
```rust
fn priority_round() {
    while consecutive_passes < 2 {
        if wants_to_pass_priority() {
            pass_priority();
            continue;
        }

        // Get all available spell abilities
        let available = self.get_available_spell_abilities(current_priority);

        // Ask controller to choose one
        let choice = controller.choose_spell_ability_to_play(&view, &available);

        if let Some(ability) = choice {
            match ability {
                SpellAbility::PlayLand { card_id } => {
                    // Land: resolve directly (no stack)
                    self.game.play_land(current_priority, card_id)?;
                }
                SpellAbility::CastSpell { card_id } => {
                    // Spell: go through 8-step process
                    self.cast_spell_with_callbacks(current_priority, card_id, controller)?;
                }
                SpellAbility::ActivateAbility { .. } => {
                    // TODO: activate ability
                }
            }
            consecutive_passes = 0;
        } else {
            // Didn't choose anything - treat as pass
            consecutive_passes += 1;
        }

        current_priority = other_player;
    }
}
```

**8-Step Casting Process (actions.rs)**:
```rust
pub fn cast_spell_with_controller(
    &mut self,
    player_id: PlayerId,
    card_id: CardId,
    controller: &mut dyn PlayerController,
) -> Result<()> {
    // Step 1: Propose - move to stack
    self.move_card(card_id, Zone::Hand, Zone::Stack, player_id)?;

    // Step 2: Make choices (TODO: modes, X values)

    // Step 3: Choose targets
    let valid_targets = self.get_valid_targets(card_id);
    let view = GameStateView::new(self, player_id);
    let targets = controller.choose_targets(&view, card_id, &valid_targets);

    // Step 4: Divide effects (TODO)

    // Step 5: Determine total cost
    let card = self.cards.get(card_id)?;
    let mana_cost = card.mana_cost.clone();

    // Step 6: Activate mana abilities
    let available_sources = self.get_mana_sources(player_id);
    let sources_to_tap = controller.choose_mana_sources_to_pay(&view, &mana_cost, &available_sources);

    // Tap the chosen sources
    for &source_id in &sources_to_tap {
        self.tap_for_mana(player_id, source_id)?;
    }

    // Step 7: Pay costs
    let player = self.get_player_mut(player_id)?;
    player.mana_pool.pay_cost(&mana_cost)?;

    // Step 8: Spell becomes cast
    // TODO: Trigger "whenever you cast" abilities

    Ok(())
}
```

## Migration Steps

1. ✅ Create SpellAbility enum in core
2. ✅ Update PlayerController trait
3. ✅ Implement 8-step casting in actions.rs
4. ✅ Update game_loop priority_round
5. ✅ Update RandomController
6. ✅ Update ZeroController
7. ✅ Update tests and examples
8. ✅ Run validation

## Benefits

- ✅ **Matches Java Forge**: Same conceptual model
- ✅ **Correct MTG Rules**: Mana tapped at proper time (step 6)
- ✅ **Unified Playing**: Lands and spells chosen from same method
- ✅ **Extensible**: Easy to add activated abilities later
- ✅ **Cleaner**: Controller just makes decisions, game enforces rules
