//! Main game state structure

use crate::core::{Card, CardId, EntityId, EntityStore, Player, PlayerId};
use crate::game::TurnStructure;
use crate::zones::{CardZone, PlayerZones, Zone};
use crate::Result;
use serde::{Deserialize, Serialize};

/// Complete game state
///
/// This is the central structure that holds all game information.
/// It's designed to be efficiently clonable for tree search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// All cards in the game
    pub cards: EntityStore<Card>,

    /// All players in the game
    pub players: EntityStore<Player>,

    /// Zones for each player
    pub player_zones: Vec<(PlayerId, PlayerZones)>,

    /// Shared battlefield (all players)
    pub battlefield: CardZone,

    /// The stack (for spells and abilities)
    pub stack: CardZone,

    /// Turn structure
    pub turn: TurnStructure,

    /// Random number generator state (for deterministic replay)
    pub rng_seed: u64,

    /// Unified entity ID generator (shared across all entity types)
    next_entity_id: u32,
}

impl GameState {
    /// Create a new game with two players
    pub fn new_two_player(player1_name: String, player2_name: String, starting_life: i32) -> Self {
        let mut next_id = 0;

        // Create players with unified IDs
        let p1_id = PlayerId::new(next_id);
        next_id += 1;
        let p2_id = PlayerId::new(next_id);
        next_id += 1;

        let player1 = Player::new(p1_id, player1_name, starting_life);
        let player2 = Player::new(p2_id, player2_name, starting_life);

        let mut players = EntityStore::new();
        players.insert(p1_id, player1);
        players.insert(p2_id, player2);

        let player_zones = vec![
            (p1_id, PlayerZones::new(p1_id)),
            (p2_id, PlayerZones::new(p2_id)),
        ];

        // Use a unified PlayerId for shared zones (battlefield, stack)
        // These don't belong to a specific player, but we need an ID for the zone
        let shared_id = PlayerId::new(next_id);
        next_id += 1;

        GameState {
            cards: EntityStore::new(),
            players,
            player_zones,
            battlefield: CardZone::new(Zone::Battlefield, shared_id),
            stack: CardZone::new(Zone::Stack, shared_id),
            turn: TurnStructure::new(p1_id),
            rng_seed: 0,
            next_entity_id: next_id,
        }
    }

    /// Get next entity ID (unified across all entity types)
    /// Generic version that can return any EntityId<T> type
    pub fn next_id<T>(&mut self) -> EntityId<T> {
        let id = EntityId::new(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }

    /// Convenience method for getting next card ID
    pub fn next_card_id(&mut self) -> CardId {
        self.next_id()
    }

    /// Convenience method for getting next player ID
    pub fn next_player_id(&mut self) -> PlayerId {
        self.next_id()
    }

    /// Legacy method for compatibility (deprecated)
    #[allow(dead_code)]
    pub fn next_entity_id(&mut self) -> CardId {
        self.next_card_id()
    }

    /// Get player zones for a specific player
    pub fn get_player_zones(&self, player_id: PlayerId) -> Option<&PlayerZones> {
        self.player_zones
            .iter()
            .find(|(id, _)| *id == player_id)
            .map(|(_, zones)| zones)
    }

    /// Get mutable player zones for a specific player
    pub fn get_player_zones_mut(&mut self, player_id: PlayerId) -> Option<&mut PlayerZones> {
        self.player_zones
            .iter_mut()
            .find(|(id, _)| *id == player_id)
            .map(|(_, zones)| zones)
    }

    /// Move a card from one zone to another
    pub fn move_card(
        &mut self,
        card_id: CardId,
        from: Zone,
        to: Zone,
        owner: PlayerId,
    ) -> Result<()> {
        // Remove from source zone
        let removed = match from {
            Zone::Battlefield => self.battlefield.remove(card_id),
            Zone::Stack => self.stack.remove(card_id),
            _ => {
                if let Some(zones) = self.get_player_zones_mut(owner) {
                    if let Some(zone) = zones.get_zone_mut(from) {
                        zone.remove(card_id)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        };

        if !removed {
            return Err(crate::MtgError::InvalidAction(format!(
                "Card {} not found in source zone",
                card_id
            )));
        }

        // Add to destination zone
        match to {
            Zone::Battlefield => self.battlefield.add(card_id),
            Zone::Stack => self.stack.add(card_id),
            _ => {
                if let Some(zones) = self.get_player_zones_mut(owner) {
                    if let Some(zone) = zones.get_zone_mut(to) {
                        zone.add(card_id);
                    }
                }
            }
        }

        Ok(())
    }

    /// Draw a card for a player
    pub fn draw_card(&mut self, player_id: PlayerId) -> Result<Option<CardId>> {
        if let Some(zones) = self.get_player_zones_mut(player_id) {
            if let Some(card_id) = zones.library.draw_top() {
                zones.hand.add(card_id);
                return Ok(Some(card_id));
            }
        }
        Ok(None)
    }

    /// Untap all permanents controlled by a player
    pub fn untap_all(&mut self, player_id: PlayerId) -> Result<()> {
        for card_id in self.battlefield.cards.iter() {
            if let Ok(card) = self.cards.get_mut(*card_id) {
                if card.controller == player_id {
                    card.untap();
                }
            }
        }
        Ok(())
    }

    /// Advance the game to the next step
    pub fn advance_step(&mut self) -> Result<()> {
        if !self.turn.advance_step() {
            // End of turn, move to next player
            let next_player = self.get_next_player(self.turn.active_player)?;
            self.turn.next_turn(next_player);

            // Reset per-turn state
            if let Ok(player) = self.players.get_mut(next_player) {
                player.reset_lands_played();
            }
        }
        Ok(())
    }

    /// Get the next player in turn order
    fn get_next_player(&self, current_player: PlayerId) -> Result<PlayerId> {
        let player_ids: Vec<PlayerId> = self.players.iter().map(|(id, _)| *id).collect();
        let current_idx = player_ids
            .iter()
            .position(|&id| id == current_player)
            .ok_or(crate::MtgError::EntityNotFound(current_player.as_u32()))?;

        let next_idx = (current_idx + 1) % player_ids.len();
        Ok(player_ids[next_idx])
    }

    /// Check if the game is over
    pub fn is_game_over(&self) -> bool {
        self.players.iter().filter(|(_, p)| !p.has_lost).count() <= 1
    }

    /// Get the winner (if game is over)
    pub fn get_winner(&self) -> Option<PlayerId> {
        if !self.is_game_over() {
            return None;
        }
        self.players
            .iter()
            .find(|(_, p)| !p.has_lost)
            .map(|(id, _)| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Step;

    #[test]
    fn test_game_creation() {
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);

        assert_eq!(game.players.len(), 2);
        assert_eq!(game.player_zones.len(), 2);
        assert_eq!(game.turn.turn_number, 1);
        assert_eq!(game.turn.current_step, Step::Untap);
    }

    #[test]
    fn test_draw_card() {
        let mut game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);

        // Create a card and add it to library
        let p1_id = *game.players.iter().next().unwrap().0; // Copy the ID
        let card_id = game.next_entity_id();
        let card = Card::new(card_id, "Test Card".to_string(), p1_id);
        game.cards.insert(card_id, card);

        // Add to library
        if let Some(zones) = game.get_player_zones_mut(p1_id) {
            zones.library.add(card_id);
        }

        // Draw the card
        let drawn = game.draw_card(p1_id).unwrap();
        assert_eq!(drawn, Some(card_id));

        // Check it's in hand
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(zones.hand.contains(card_id));
            assert!(!zones.library.contains(card_id));
        }
    }

    #[test]
    fn test_game_over() {
        let mut game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);

        assert!(!game.is_game_over());
        assert_eq!(game.get_winner(), None);

        // Make player 1 lose
        let p1_id = *game.players.iter().next().unwrap().0; // Copy the ID
        if let Ok(player) = game.players.get_mut(p1_id) {
            player.lose_life(20);
        }

        assert!(game.is_game_over());
        let winner = game.get_winner().unwrap();
        assert_ne!(winner, p1_id);
    }
}
