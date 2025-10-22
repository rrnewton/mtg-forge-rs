//! Game actions and mechanics

use crate::core::{CardId, CardType, Effect, Keyword, PlayerId, TargetRef};
use crate::game::GameState;
use crate::zones::Zone;
use crate::{MtgError, Result};

/// Types of game actions
#[derive(Debug, Clone)]
pub enum GameAction {
    /// Play a land from hand
    PlayLand {
        player_id: PlayerId,
        card_id: CardId,
    },

    /// Cast a spell from hand
    CastSpell {
        player_id: PlayerId,
        card_id: CardId,
        targets: Vec<CardId>,
    },

    /// Deal damage to a target
    DealDamage {
        source: CardId,
        target: CardId,
        amount: i32,
    },

    /// Tap a permanent for mana
    TapForMana {
        player_id: PlayerId,
        card_id: CardId,
    },

    /// Declare attackers
    DeclareAttackers {
        player_id: PlayerId,
        attackers: Vec<CardId>,
    },

    /// Pass priority
    PassPriority { player_id: PlayerId },
}

impl GameState {
    /// Play a land from hand to battlefield
    pub fn play_land(&mut self, player_id: PlayerId, card_id: CardId) -> Result<()> {
        // Check if player can play a land
        let player = self.get_player(player_id)?;
        if !player.can_play_land() {
            return Err(MtgError::InvalidAction(
                "Cannot play more lands this turn".to_string(),
            ));
        }

        // Check if card is a land and in hand
        let card = self.cards.get(card_id)?;
        if !card.is_land() {
            return Err(MtgError::InvalidAction("Card is not a land".to_string()));
        }

        // Check if in hand
        if let Some(zones) = self.get_player_zones(player_id) {
            if !zones.hand.contains(card_id) {
                return Err(MtgError::InvalidAction("Card not in hand".to_string()));
            }
        }

        // Move card to battlefield
        self.move_card(card_id, Zone::Hand, Zone::Battlefield, player_id)?;

        // Record the turn number when this land entered the battlefield
        if let Ok(card) = self.cards.get_mut(card_id) {
            card.turn_entered_battlefield = Some(self.turn.turn_number);
        }

        // Increment lands played
        let player = self.get_player_mut(player_id)?;
        player.play_land();

        Ok(())
    }

    /// Cast a spell (put it on the stack)
    ///
    /// This validates mana payment and deducts the cost from the player's mana pool.
    pub fn cast_spell(
        &mut self,
        player_id: PlayerId,
        card_id: CardId,
        _targets: Vec<CardId>,
    ) -> Result<()> {
        // Check if card is in hand
        if let Some(zones) = self.get_player_zones(player_id) {
            if !zones.hand.contains(card_id) {
                return Err(MtgError::InvalidAction("Card not in hand".to_string()));
            }
        }

        // Get the mana cost (need to do this before mutable borrow)
        let mana_cost = {
            let card = self.cards.get(card_id)?;
            card.mana_cost.clone()
        };

        // Pay the mana cost
        let player = self.get_player_mut(player_id)?;
        player
            .mana_pool
            .pay_cost(&mana_cost)
            .map_err(MtgError::InvalidAction)?;

        // Move card to stack
        self.move_card(card_id, Zone::Hand, Zone::Stack, player_id)?;

        Ok(())
    }

    /// Resolve a spell from the stack
    pub fn resolve_spell(&mut self, card_id: CardId) -> Result<()> {
        // Get card owner and effects
        let (card_owner, mut effects) = {
            let card = self.cards.get(card_id)?;
            (card.owner, card.effects.clone())
        };

        // Fill in missing targets for effects
        // For now, target an opponent for DealDamage effects with no target
        for effect in &mut effects {
            if let Effect::DealDamage {
                target: TargetRef::None,
                amount,
            } = effect
            {
                // Find an opponent
                if let Some(opponent_id) = self
                    .players
                    .iter()
                    .map(|p| p.id)
                    .find(|id| *id != card_owner)
                {
                    *effect = Effect::DealDamage {
                        target: TargetRef::Player(opponent_id),
                        amount: *amount,
                    };
                }
            }
        }

        for effect in effects {
            self.execute_effect(&effect)?;
        }

        // Determine destination based on card type
        let destination = {
            let card = self.cards.get(card_id)?;
            if card.is_type(&CardType::Instant) || card.is_type(&CardType::Sorcery) {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        };

        // Move card from stack to destination
        let owner = self.cards.get(card_id)?.owner;
        self.move_card(card_id, Zone::Stack, destination, owner)?;

        // If it entered the battlefield, record the turn number (for summoning sickness)
        if destination == Zone::Battlefield {
            if let Ok(card) = self.cards.get_mut(card_id) {
                card.turn_entered_battlefield = Some(self.turn.turn_number);
            }
        }

        Ok(())
    }

    /// Cast a spell following the full 8-step process (MTG Rules 601.2)
    ///
    /// This method implements the complete spell casting sequence:
    /// 1. Propose the spell (move to stack)
    /// 2. Make choices (modes, X values) - TODO
    /// 3. Choose targets
    /// 4. Divide effects - TODO
    /// 5. Determine total cost
    /// 6. Activate mana abilities (tap sources for mana)
    /// 7. Pay costs
    /// 8. Spell becomes cast (trigger abilities) - TODO
    ///
    /// ## Parameters
    /// - `player_id`: The player casting the spell
    /// - `card_id`: The spell card to cast
    /// - `choose_targets_fn`: Callback to choose targets (step 3)
    /// - `choose_mana_sources_fn`: Callback to choose what to tap for mana (step 6)
    ///
    /// ## Java Forge Equivalent
    /// This matches `ComputerUtil.handlePlayingSpellAbility()` which:
    /// 1. Moves spell to stack (line 99)
    /// 2. Handles targeting
    /// 3. Pays costs with `CostPayment.payComputerCosts()` (line 125)
    pub fn cast_spell_8_step<TargetFn, ManaFn>(
        &mut self,
        player_id: PlayerId,
        card_id: CardId,
        mut choose_targets_fn: TargetFn,
        mut choose_mana_sources_fn: ManaFn,
    ) -> Result<()>
    where
        TargetFn: FnMut(&GameState, CardId) -> Vec<CardId>,
        ManaFn: FnMut(&GameState, &crate::core::ManaCost) -> Vec<CardId>,
    {
        // Verify card is in hand
        if let Some(zones) = self.get_player_zones(player_id) {
            if !zones.hand.contains(card_id) {
                return Err(MtgError::InvalidAction("Card not in hand".to_string()));
            }
        }

        // Step 1: Propose the spell - move card to stack
        // This happens BEFORE paying costs (unlike our old implementation)
        self.move_card(card_id, Zone::Hand, Zone::Stack, player_id)?;

        // Step 2: Make choices (modes, X values)
        // TODO: Implement modal spell choices and X value selection

        // Step 3: Choose targets
        let _targets = choose_targets_fn(self, card_id);
        // TODO: Store targets on the spell for resolution
        // For now, we'll use them to update effects immediately (simplified)

        // Step 4: Divide effects
        // TODO: Implement dividing damage/counters among targets

        // Step 5: Determine total cost
        let mana_cost = {
            let card = self.cards.get(card_id)?;
            card.mana_cost.clone()
        };

        // Step 6: Activate mana abilities
        // This is where mana gets tapped - AFTER the spell is on the stack
        let sources_to_tap = choose_mana_sources_fn(self, &mana_cost);
        for &source_id in &sources_to_tap {
            self.tap_for_mana(player_id, source_id)?;
        }

        // Step 7: Pay costs
        let player = self.get_player_mut(player_id)?;
        if let Err(e) = player.mana_pool.pay_cost(&mana_cost) {
            // If we can't pay, we need to unwind - move card back to hand
            // and untap mana sources
            // For now, just return error (TODO: proper unwinding)
            return Err(MtgError::InvalidAction(format!(
                "Failed to pay mana cost: {}",
                e
            )));
        }

        // Step 8: Spell becomes cast
        // TODO: Trigger "whenever you cast a spell" abilities
        // For now, this is complete - spell is on stack and costs are paid

        Ok(())
    }

    /// Execute a single effect
    pub fn execute_effect(&mut self, effect: &Effect) -> Result<()> {
        match effect {
            Effect::DealDamage { target, amount } => match target {
                TargetRef::Player(player_id) => {
                    self.deal_damage(*player_id, *amount)?;
                }
                TargetRef::Permanent(card_id) => {
                    self.deal_damage_to_creature(*card_id, *amount)?;
                }
                TargetRef::None => {
                    return Err(MtgError::InvalidAction(
                        "DealDamage effect requires a target".to_string(),
                    ));
                }
            },
            Effect::DrawCards { player, count } => {
                for _ in 0..*count {
                    self.draw_card(*player)?;
                }
            }
            Effect::GainLife { player, amount } => {
                let p = self.get_player_mut(*player)?;
                p.gain_life(*amount);

                // Log the life gain
                self.undo_log.log(crate::undo::GameAction::ModifyLife {
                    player_id: *player,
                    delta: *amount,
                });
            }
            Effect::DestroyPermanent { target } => {
                let owner = self.cards.get(*target)?.owner;
                self.move_card(*target, Zone::Battlefield, Zone::Graveyard, owner)?;
            }
            Effect::TapPermanent { target } => {
                let card = self.cards.get_mut(*target)?;
                card.tap();

                // Log the tap
                self.undo_log.log(crate::undo::GameAction::TapCard {
                    card_id: *target,
                    tapped: true,
                });
            }
            Effect::UntapPermanent { target } => {
                let card = self.cards.get_mut(*target)?;
                card.untap();

                // Log the untap
                self.undo_log.log(crate::undo::GameAction::TapCard {
                    card_id: *target,
                    tapped: false,
                });
            }
        }
        Ok(())
    }

    /// Deal damage to a player target
    pub fn deal_damage(&mut self, target_id: PlayerId, amount: i32) -> Result<()> {
        // Check if target is a player
        if self.players.iter().any(|p| p.id == target_id) {
            let player = self.get_player_mut(target_id)?;
            player.lose_life(amount);

            // Log the life change
            self.undo_log.log(crate::undo::GameAction::ModifyLife {
                player_id: target_id,
                delta: -amount,
            });

            return Ok(());
        }

        Err(MtgError::InvalidAction("Invalid damage target".to_string()))
    }

    /// Deal damage to a creature
    pub fn deal_damage_to_creature(&mut self, target_id: CardId, amount: i32) -> Result<()> {
        // Get info about the creature first (without holding the borrow)
        let (is_creature, toughness, owner) = {
            let card = self.cards.get(target_id)?;
            (card.is_creature(), card.current_toughness(), card.owner)
        };

        if is_creature {
            // Mark damage (simplified - real MTG has damage tracking)
            // For now, if damage >= toughness, creature dies
            if amount >= toughness as i32 {
                self.move_card(target_id, Zone::Battlefield, Zone::Graveyard, owner)?;
            }
            return Ok(());
        }

        Err(MtgError::InvalidAction("Invalid damage target".to_string()))
    }

    /// Tap a land for mana
    pub fn tap_for_mana(&mut self, player_id: PlayerId, card_id: CardId) -> Result<()> {
        let card = self.cards.get_mut(card_id)?;

        // Check if card is a land and untapped
        if !card.is_land() {
            return Err(MtgError::InvalidAction("Card is not a land".to_string()));
        }

        if card.tapped {
            return Err(MtgError::InvalidAction(
                "Land is already tapped".to_string(),
            ));
        }

        // Get land name before tapping (to avoid borrow conflicts)
        let land_name = card.name.to_lowercase();

        // Tap the land
        card.tap();

        // Log the tap
        self.undo_log.log(crate::undo::GameAction::TapCard {
            card_id,
            tapped: true,
        });

        // Add mana to player's pool based on land type
        // For now, simplified - just check land name
        let player = self.get_player_mut(player_id)?;
        let color = if land_name.contains("mountain") {
            Some(crate::core::Color::Red)
        } else if land_name.contains("island") {
            Some(crate::core::Color::Blue)
        } else if land_name.contains("swamp") {
            Some(crate::core::Color::Black)
        } else if land_name.contains("forest") {
            Some(crate::core::Color::Green)
        } else if land_name.contains("plains") {
            Some(crate::core::Color::White)
        } else {
            None
        };

        if let Some(color) = color {
            player.mana_pool.add_color(color);

            // Log the mana addition
            self.undo_log
                .log(crate::undo::GameAction::AddMana { player_id, color });
        }

        Ok(())
    }

    /// Declare a creature as an attacker
    pub fn declare_attacker(&mut self, player_id: PlayerId, card_id: CardId) -> Result<()> {
        // Validate creature can attack
        let card = self.cards.get(card_id)?;

        // Must be a creature
        if !card.is_creature() {
            return Err(MtgError::InvalidAction(
                "Only creatures can attack".to_string(),
            ));
        }

        // Must be controlled by the attacking player
        if card.controller != player_id {
            return Err(MtgError::InvalidAction(
                "Can't attack with opponent's creatures".to_string(),
            ));
        }

        // Must be on battlefield
        if !self.battlefield.contains(card_id) {
            return Err(MtgError::InvalidAction(
                "Creature must be on battlefield to attack".to_string(),
            ));
        }

        // Must not be tapped
        if card.tapped {
            return Err(MtgError::InvalidAction(
                "Creature is tapped and can't attack".to_string(),
            ));
        }

        // Check for summoning sickness
        // Creatures can't attack the turn they entered the battlefield unless they have haste
        if let Some(entered_turn) = card.turn_entered_battlefield {
            if entered_turn == self.turn.turn_number && !card.has_keyword(&Keyword::Haste) {
                return Err(MtgError::InvalidAction(
                    "Creature has summoning sickness and can't attack this turn".to_string(),
                ));
            }
        }

        // Get defending player (for 2-player, it's the other player)
        let defending_player = self
            .players
            .iter()
            .find(|p| p.id != player_id)
            .map(|p| p.id)
            .ok_or_else(|| MtgError::InvalidAction("No opponent found".to_string()))?;

        // Declare attacker in combat state
        self.combat.declare_attacker(card_id, defending_player);

        // Tap the creature (unless it has vigilance)
        let has_vigilance = self.cards.get(card_id)?.has_keyword(&Keyword::Vigilance);
        if !has_vigilance {
            let card = self.cards.get_mut(card_id)?;
            card.tap();

            // Log the action
            self.undo_log.log(crate::undo::GameAction::TapCard {
                card_id,
                tapped: true,
            });
        }

        Ok(())
    }

    /// Declare a creature as a blocker
    pub fn declare_blocker(
        &mut self,
        player_id: PlayerId,
        blocker_id: CardId,
        attackers: Vec<CardId>,
    ) -> Result<()> {
        // Validate blocker can block
        let blocker = self.cards.get(blocker_id)?;

        // Must be a creature
        if !blocker.is_creature() {
            return Err(MtgError::InvalidAction(
                "Only creatures can block".to_string(),
            ));
        }

        // Must be controlled by the defending player
        if blocker.controller != player_id {
            return Err(MtgError::InvalidAction(
                "Can't block with opponent's creatures".to_string(),
            ));
        }

        // Must be on battlefield
        if !self.battlefield.contains(blocker_id) {
            return Err(MtgError::InvalidAction(
                "Creature must be on battlefield to block".to_string(),
            ));
        }

        // Must not be tapped
        if blocker.tapped {
            return Err(MtgError::InvalidAction(
                "Creature is tapped and can't block".to_string(),
            ));
        }

        // Validate all attackers are actually attacking
        for &attacker in &attackers {
            if !self.combat.is_attacking(attacker) {
                return Err(MtgError::InvalidAction(format!(
                    "Card {:?} is not attacking",
                    attacker
                )));
            }
        }

        // MTG rule: normally a creature can only block one attacker
        // (unless it has an ability that allows it to block multiple)
        if attackers.len() > 1 {
            // TODO: Check for abilities that allow blocking multiple
            return Err(MtgError::InvalidAction(
                "Creature can only block one attacker".to_string(),
            ));
        }

        // Declare blocker
        let mut attackers_vec = smallvec::SmallVec::new();
        for &attacker in &attackers {
            attackers_vec.push(attacker);
        }
        self.combat.declare_blocker(blocker_id, attackers_vec);

        Ok(())
    }

    /// Assign and deal combat damage
    pub fn assign_combat_damage(&mut self) -> Result<()> {
        // Get all attackers
        let attackers = self.combat.get_attackers();

        for attacker_id in attackers {
            let attacker = self.cards.get(attacker_id)?;
            let power = attacker.current_power();

            if power <= 0 {
                continue; // 0 or negative power deals no damage
            }

            // Check if attacker is blocked
            if self.combat.is_blocked(attacker_id) {
                // Attacker deals damage to blockers
                let blockers = self.combat.get_blockers(attacker_id);

                // For now, simplified: distribute damage evenly among blockers
                // TODO: Allow attacker's controller to order blockers and assign damage
                let damage_per_blocker = power / blockers.len() as i8;

                for blocker_id in &blockers {
                    if damage_per_blocker > 0 {
                        self.deal_damage_to_creature(*blocker_id, damage_per_blocker as i32)?;
                    }

                    // Blocker deals damage back to attacker
                    let blocker = self.cards.get(*blocker_id)?;
                    let blocker_power = blocker.current_power();
                    if blocker_power > 0 {
                        self.deal_damage_to_creature(attacker_id, blocker_power as i32)?;
                    }
                }
            } else {
                // Unblocked attacker deals damage to defending player
                if let Some(defending_player) = self.combat.get_defending_player(attacker_id) {
                    self.deal_damage(defending_player, power as i32)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Card;

    #[test]
    fn test_play_land() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Create a mountain card
        let card_id = game.next_entity_id();
        let mut card = Card::new(card_id, "Mountain".to_string(), p1_id);
        card.types.push(CardType::Land);
        game.cards.insert(card_id, card);

        // Add to hand
        if let Some(zones) = game.get_player_zones_mut(p1_id) {
            zones.hand.add(card_id);
        }

        // Play the land
        assert!(game.play_land(p1_id, card_id).is_ok());

        // Check it's on battlefield
        assert!(game.battlefield.contains(card_id));

        // Check player used their land drop
        let player = game.get_player(p1_id).unwrap();
        assert!(!player.can_play_land());
    }

    #[test]
    fn test_tap_for_mana() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Create a mountain on battlefield
        let card_id = game.next_entity_id();
        let mut card = Card::new(card_id, "Mountain".to_string(), p1_id);
        card.types.push(CardType::Land);
        game.cards.insert(card_id, card);
        game.battlefield.add(card_id);

        // Tap for mana
        assert!(game.tap_for_mana(p1_id, card_id).is_ok());

        // Check mana was added
        let player = game.get_player(p1_id).unwrap();
        assert_eq!(player.mana_pool.red, 1);

        // Check land is tapped
        let card = game.cards.get(card_id).unwrap();
        assert!(card.tapped);
    }

    #[test]
    fn test_deal_damage_to_player() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Deal 3 damage
        assert!(game.deal_damage(p1_id, 3).is_ok());

        let player = game.get_player(p1_id).unwrap();
        assert_eq!(player.life, 17);
    }

    #[test]
    fn test_move_card_battlefield_to_graveyard() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Create a creature on battlefield
        let card_id = game.next_entity_id();
        let card = Card::new(card_id, "Test Card".to_string(), p1_id);
        game.cards.insert(card_id, card);
        game.battlefield.add(card_id);

        // Test move_card directly
        let result = game.move_card(card_id, Zone::Battlefield, Zone::Graveyard, p1_id);
        if let Err(e) = &result {
            panic!("move_card failed: {e:?}");
        }

        // Check it moved
        assert!(
            !game.battlefield.contains(card_id),
            "Card still on battlefield"
        );
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(zones.graveyard.contains(card_id), "Card not in graveyard");
        }
    }

    #[test]
    fn test_deal_damage_to_creature() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Create a 2/2 creature on battlefield
        let card_id = game.next_card_id();
        let mut card = Card::new(card_id, "Grizzly Bears".to_string(), p1_id);
        card.types.push(CardType::Creature);
        card.power = Some(2);
        card.toughness = Some(2);
        game.cards.insert(card_id, card);
        game.battlefield.add(card_id);

        // Deal 2 damage (should kill it)
        let result = game.deal_damage_to_creature(card_id, 2);
        assert!(result.is_ok(), "deal_damage_to_creature failed: {result:?}");

        // Check it's in graveyard
        assert!(
            !game.battlefield.contains(card_id),
            "Card still on battlefield"
        );
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(zones.graveyard.contains(card_id), "Card not in graveyard");
        }
    }

    #[test]
    fn test_cast_spell_with_mana_payment() {
        use crate::core::{Color, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players.first().unwrap().id;

        // Create a Lightning Bolt in hand (cost: R)
        let bolt_id = game.next_card_id();
        let mut bolt = Card::new(bolt_id, "Lightning Bolt".to_string(), p1_id);
        bolt.types.push(CardType::Instant);
        bolt.mana_cost = ManaCost::from_string("R");
        game.cards.insert(bolt_id, bolt);

        // Add to hand
        if let Some(zones) = game.get_player_zones_mut(p1_id) {
            zones.hand.add(bolt_id);
        }

        // Try to cast without mana - should fail
        let result = game.cast_spell(p1_id, bolt_id, vec![]);
        assert!(result.is_err());

        // Add mana to pool
        let player = game.get_player_mut(p1_id).unwrap();
        player.mana_pool.add_color(Color::Red);

        // Now cast should succeed
        let result = game.cast_spell(p1_id, bolt_id, vec![]);
        assert!(result.is_ok(), "cast_spell failed: {result:?}");

        // Check mana was deducted
        let player = game.get_player(p1_id).unwrap();
        assert_eq!(player.mana_pool.red, 0);

        // Check card is on stack
        assert!(game.stack.contains(bolt_id));
    }

    #[test]
    fn test_cast_spell_with_generic_mana() {
        use crate::core::{Color, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players.first().unwrap().id;

        // Create a spell with cost 2R
        let spell_id = game.next_card_id();
        let mut spell = Card::new(spell_id, "Lava Spike".to_string(), p1_id);
        spell.types.push(CardType::Sorcery);
        spell.mana_cost = ManaCost::from_string("2R");
        game.cards.insert(spell_id, spell);

        // Add to hand
        if let Some(zones) = game.get_player_zones_mut(p1_id) {
            zones.hand.add(spell_id);
        }

        // Add mana: 2R + 1U = 4 mana total
        let player = game.get_player_mut(p1_id).unwrap();
        player.mana_pool.add_color(Color::Red);
        player.mana_pool.add_color(Color::Red);
        player.mana_pool.add_color(Color::Blue);

        // Cast spell - should use 1R for R, and 2R for generic 2
        let result = game.cast_spell(p1_id, spell_id, vec![]);
        assert!(result.is_ok(), "cast_spell failed: {result:?}");

        // Check mana was deducted properly (should have 1 blue left)
        let player = game.get_player(p1_id).unwrap();
        assert_eq!(player.mana_pool.red, 0);
        assert_eq!(player.mana_pool.blue, 0); // Blue was used for generic cost
        assert_eq!(player.mana_pool.total(), 0);

        // Check card is on stack
        assert!(game.stack.contains(spell_id));
    }

    #[test]
    fn test_execute_damage_effect_to_player() {
        use crate::core::{Effect, TargetRef};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p2_id = players[1];

        let effect = Effect::DealDamage {
            target: TargetRef::Player(p2_id),
            amount: 3,
        };

        assert!(game.execute_effect(&effect).is_ok());

        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(p2.life, 17);
    }

    #[test]
    fn test_execute_draw_effect() {
        use crate::core::Effect;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players.first().unwrap().id;

        // Add cards to library
        for i in 0..5 {
            let card_id = game.next_card_id();
            let card = Card::new(card_id, format!("Card {i}"), p1_id);
            game.cards.insert(card_id, card);
            if let Some(zones) = game.get_player_zones_mut(p1_id) {
                zones.library.add(card_id);
            }
        }

        let effect = Effect::DrawCards {
            player: p1_id,
            count: 2,
        };

        assert!(game.execute_effect(&effect).is_ok());

        // Check cards were drawn
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert_eq!(zones.hand.cards.len(), 2);
            assert_eq!(zones.library.cards.len(), 3);
        }
    }

    #[test]
    fn test_resolve_spell_with_effects() {
        use crate::core::{Effect, ManaCost, TargetRef};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create Lightning Bolt with damage effect
        let bolt_id = game.next_card_id();
        let mut bolt = Card::new(bolt_id, "Lightning Bolt".to_string(), p1_id);
        bolt.types.push(CardType::Instant);
        bolt.mana_cost = ManaCost::from_string("R");
        bolt.effects.push(Effect::DealDamage {
            target: TargetRef::Player(p2_id),
            amount: 3,
        });
        game.cards.insert(bolt_id, bolt);

        // Put it on the stack (simulating cast)
        game.stack.add(bolt_id);

        // Resolve the spell
        assert!(game.resolve_spell(bolt_id).is_ok());

        // Check damage was dealt
        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(p2.life, 17);

        // Check spell went to graveyard
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(zones.graveyard.contains(bolt_id));
        }
    }

    #[test]
    fn test_declare_attacker() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];

        // Create a creature
        let creature_id = game.next_card_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);

        // Put creature on battlefield
        game.battlefield.add(creature_id);

        // Declare attacker
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok(), "Failed to declare attacker: {result:?}");

        // Check creature is attacking
        assert!(game.combat.is_attacking(creature_id));

        // Check creature is tapped
        let creature = game.cards.get(creature_id).unwrap();
        assert!(creature.tapped);
    }

    #[test]
    fn test_declare_blocker() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create an attacker
        let attacker_id = game.next_card_id();
        let mut attacker = Card::new(attacker_id, "Goblin".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(2);
        attacker.toughness = Some(1);
        attacker.controller = p1_id;
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // Declare as attacker
        game.combat.declare_attacker(attacker_id, p2_id);

        // Create a blocker
        let blocker_id = game.next_card_id();
        let mut blocker = Card::new(blocker_id, "Wall".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(0);
        blocker.toughness = Some(3);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare blocker
        let result = game.declare_blocker(p2_id, blocker_id, vec![attacker_id]);
        assert!(result.is_ok(), "Failed to declare blocker: {result:?}");

        // Check blocker is blocking
        assert!(game.combat.is_blocking(blocker_id));
        assert!(game.combat.is_blocked(attacker_id));

        let blockers = game.combat.get_blockers(attacker_id);
        assert_eq!(blockers.len(), 1);
        assert!(blockers.contains(&blocker_id));
    }

    #[test]
    fn test_combat_damage_unblocked() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create an attacker
        let attacker_id = game.next_card_id();
        let mut attacker = Card::new(attacker_id, "Dragon".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(5);
        attacker.toughness = Some(5);
        attacker.controller = p1_id;
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // Declare as attacker (unblocked)
        game.combat.declare_attacker(attacker_id, p2_id);

        // Assign combat damage
        let result = game.assign_combat_damage();
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Check defending player took damage
        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(p2.life, 15); // 20 - 5 = 15
    }

    #[test]
    fn test_combat_damage_blocked() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create an attacker (3/3)
        let attacker_id = game.next_card_id();
        let mut attacker = Card::new(attacker_id, "Bear".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(3);
        attacker.toughness = Some(3);
        attacker.controller = p1_id;
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // Create a blocker (2/2)
        let blocker_id = game.next_card_id();
        let mut blocker = Card::new(blocker_id, "Wolf".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(2);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare attacker and blocker
        game.combat.declare_attacker(attacker_id, p2_id);
        let blocker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, blocker_vec);

        // Assign combat damage
        let result = game.assign_combat_damage();
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Check defending player took no damage (blocked)
        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(p2.life, 20);

        // Check blocker died (took 3 damage, toughness 2)
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(zones.graveyard.contains(blocker_id));
        }

        // Check attacker died (took 2 damage, toughness 2... wait, 3-2=1, so it should survive)
        // Actually the attacker took 2 damage but has toughness 3, so it survives
        assert!(game.battlefield.contains(attacker_id));
    }

    #[test]
    fn test_summoning_sickness_blocks_attack() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a creature and put it on battlefield
        let creature_id = game.next_entity_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Mark it as entering this turn (summoning sickness)
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number);
        }

        // Try to declare it as an attacker - should fail due to summoning sickness
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("summoning sickness"));
    }

    #[test]
    fn test_summoning_sickness_allows_attack_next_turn() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a creature and put it on battlefield
        let creature_id = game.next_entity_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Mark it as entering on a previous turn
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }

        // Declare it as an attacker - should succeed
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok());
        assert!(game.combat.is_attacking(creature_id));
    }

    #[test]
    fn test_haste_bypasses_summoning_sickness() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a creature with haste
        let creature_id = game.next_entity_id();
        let mut creature = Card::new(creature_id, "Lightning Elemental".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(4);
        creature.toughness = Some(1);
        creature.controller = p1_id;
        creature.keywords.push(Keyword::Haste);
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Mark it as entering this turn
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number);
        }

        // Declare it as an attacker - should succeed because of haste
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok());
        assert!(game.combat.is_attacking(creature_id));
    }

    #[test]
    fn test_vigilance_creature_stays_untapped() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a creature with vigilance
        let creature_id = game.next_entity_id();
        let mut creature = Card::new(creature_id, "Serra Angel".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(4);
        creature.toughness = Some(4);
        creature.controller = p1_id;
        creature.keywords.push(Keyword::Vigilance);
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Mark it as entering on a previous turn (no summoning sickness)
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }

        // Declare it as an attacker
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok());
        assert!(game.combat.is_attacking(creature_id));

        // Check that creature is still untapped (vigilance effect)
        let card = game.cards.get(creature_id).unwrap();
        assert!(
            !card.tapped,
            "Creature with vigilance should not be tapped after attacking"
        );
    }

    #[test]
    fn test_non_vigilance_creature_gets_tapped() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a creature WITHOUT vigilance
        let creature_id = game.next_entity_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Mark it as entering on a previous turn (no summoning sickness)
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }

        // Declare it as an attacker
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok());
        assert!(game.combat.is_attacking(creature_id));

        // Check that creature is tapped (normal attack behavior)
        let card = game.cards.get(creature_id).unwrap();
        assert!(
            card.tapped,
            "Creature without vigilance should be tapped after attacking"
        );
    }
}
