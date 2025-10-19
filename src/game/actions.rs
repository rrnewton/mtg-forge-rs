//! Game actions and mechanics

use crate::core::{CardType, EntityId};
use crate::game::GameState;
use crate::zones::Zone;
use crate::{MtgError, Result};

/// Types of game actions
#[derive(Debug, Clone)]
pub enum GameAction {
    /// Play a land from hand
    PlayLand {
        player_id: EntityId,
        card_id: EntityId,
    },

    /// Cast a spell from hand
    CastSpell {
        player_id: EntityId,
        card_id: EntityId,
        targets: Vec<EntityId>,
    },

    /// Deal damage to a target
    DealDamage {
        source: EntityId,
        target: EntityId,
        amount: i32,
    },

    /// Tap a permanent for mana
    TapForMana {
        player_id: EntityId,
        card_id: EntityId,
    },

    /// Declare attackers
    DeclareAttackers {
        player_id: EntityId,
        attackers: Vec<EntityId>,
    },

    /// Pass priority
    PassPriority { player_id: EntityId },
}

impl GameState {
    /// Play a land from hand to battlefield
    pub fn play_land(&mut self, player_id: EntityId, card_id: EntityId) -> Result<()> {
        // Check if player can play a land
        let player = self.players.get(player_id)?;
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

        // Increment lands played
        let player = self.players.get_mut(player_id)?;
        player.play_land();

        Ok(())
    }

    /// Cast a spell (put it on the stack)
    pub fn cast_spell(
        &mut self,
        player_id: EntityId,
        card_id: EntityId,
        _targets: Vec<EntityId>,
    ) -> Result<()> {
        // Check if card is in hand
        if let Some(zones) = self.get_player_zones(player_id) {
            if !zones.hand.contains(card_id) {
                return Err(MtgError::InvalidAction("Card not in hand".to_string()));
            }
        }

        let card = self.cards.get(card_id)?;

        // Check if player can pay mana cost
        let player = self.players.get(player_id)?;
        if !player.mana_pool.can_pay(&card.mana_cost) {
            return Err(MtgError::InvalidAction("Cannot pay mana cost".to_string()));
        }

        // TODO: Actually pay the mana cost (requires mana payment logic)

        // Move card to stack
        self.move_card(card_id, Zone::Hand, Zone::Stack, player_id)?;

        Ok(())
    }

    /// Resolve a spell from the stack
    pub fn resolve_spell(&mut self, card_id: EntityId) -> Result<()> {
        let card = self.cards.get(card_id)?;
        let owner = card.owner;

        // Determine destination based on card type
        let destination = if card.is_type(&CardType::Instant) || card.is_type(&CardType::Sorcery) {
            Zone::Graveyard
        } else {
            Zone::Battlefield
        };

        // Move card from stack to destination
        self.move_card(card_id, Zone::Stack, destination, owner)?;

        Ok(())
    }

    /// Deal damage to a target
    pub fn deal_damage(&mut self, target_id: EntityId, amount: i32) -> Result<()> {
        // Check if target is a player
        if self.players.contains(target_id) {
            let player = self.players.get_mut(target_id)?;
            player.lose_life(amount);
            return Ok(());
        }

        // Check if target is a creature
        if self.cards.contains(target_id) {
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
        }

        Err(MtgError::InvalidAction("Invalid damage target".to_string()))
    }

    /// Tap a land for mana
    pub fn tap_for_mana(&mut self, player_id: EntityId, card_id: EntityId) -> Result<()> {
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

        // Tap the land
        card.tap();

        // Add mana to player's pool based on land type
        // For now, simplified - just check land name
        let player = self.players.get_mut(player_id)?;

        let land_name = card.name.to_lowercase();
        if land_name.contains("mountain") {
            player.mana_pool.add_color(crate::core::Color::Red);
        } else if land_name.contains("island") {
            player.mana_pool.add_color(crate::core::Color::Blue);
        } else if land_name.contains("swamp") {
            player.mana_pool.add_color(crate::core::Color::Black);
        } else if land_name.contains("forest") {
            player.mana_pool.add_color(crate::core::Color::Green);
        } else if land_name.contains("plains") {
            player.mana_pool.add_color(crate::core::Color::White);
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

        let p1_id = *game.players.iter().next().unwrap().0;

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
        let player = game.players.get(p1_id).unwrap();
        assert!(!player.can_play_land());
    }

    #[test]
    fn test_tap_for_mana() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = *game.players.iter().next().unwrap().0;

        // Create a mountain on battlefield
        let card_id = game.next_entity_id();
        let mut card = Card::new(card_id, "Mountain".to_string(), p1_id);
        card.types.push(CardType::Land);
        game.cards.insert(card_id, card);
        game.battlefield.add(card_id);

        // Tap for mana
        assert!(game.tap_for_mana(p1_id, card_id).is_ok());

        // Check mana was added
        let player = game.players.get(p1_id).unwrap();
        assert_eq!(player.mana_pool.red, 1);

        // Check land is tapped
        let card = game.cards.get(card_id).unwrap();
        assert!(card.tapped);
    }

    #[test]
    fn test_deal_damage_to_player() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = *game.players.iter().next().unwrap().0;

        // Deal 3 damage
        assert!(game.deal_damage(p1_id, 3).is_ok());

        let player = game.players.get(p1_id).unwrap();
        assert_eq!(player.life, 17);
    }

    #[test]
    fn test_move_card_battlefield_to_graveyard() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = *game.players.iter().next().unwrap().0;

        // Create a creature on battlefield
        let card_id = game.next_entity_id();
        let card = Card::new(card_id, "Test Card".to_string(), p1_id);
        game.cards.insert(card_id, card);
        game.battlefield.add(card_id);

        // Test move_card directly
        let result = game.move_card(card_id, Zone::Battlefield, Zone::Graveyard, p1_id);
        if let Err(e) = &result {
            panic!("move_card failed: {:?}", e);
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

        let p1_id = *game.players.iter().next().unwrap().0;

        // Create a 2/2 creature on battlefield
        let card_id = game.next_entity_id();
        let mut card = Card::new(card_id, "Grizzly Bears".to_string(), p1_id);
        card.types.push(CardType::Creature);
        card.power = Some(2);
        card.toughness = Some(2);
        game.cards.insert(card_id, card);
        game.battlefield.add(card_id);

        // Deal 2 damage (should kill it)
        let result = game.deal_damage(card_id, 2);
        assert!(result.is_ok(), "deal_damage failed: {:?}", result);

        // Check it's in graveyard
        assert!(
            !game.battlefield.contains(card_id),
            "Card still on battlefield"
        );
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(zones.graveyard.contains(card_id), "Card not in graveyard");
        }
    }
}
