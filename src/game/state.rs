//! Main game state structure

use crate::core::{Card, CardId, EntityId, EntityStore, Player, PlayerId};
use crate::game::{CombatState, TurnStructure};
use crate::undo::UndoLog;
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

    /// All players in the game (Vec for stable ordering, small count)
    pub players: Vec<Player>,

    /// Zones for each player
    pub player_zones: Vec<(PlayerId, PlayerZones)>,

    /// Shared battlefield (all players)
    pub battlefield: CardZone,

    /// The stack (for spells and abilities)
    pub stack: CardZone,

    /// Turn structure
    pub turn: TurnStructure,

    /// Combat state (active during combat phase)
    pub combat: CombatState,

    /// Random number generator state (for deterministic replay)
    pub rng_seed: u64,

    /// Unified entity ID generator (shared across all entity types)
    next_entity_id: u32,

    /// Undo log for tracking all game actions
    pub undo_log: UndoLog,
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

        let players = vec![player1, player2];

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
            turn: TurnStructure::new_with_idx(p1_id, 0), // Player 1 starts at index 0
            combat: CombatState::new(),
            rng_seed: 0,
            next_entity_id: next_id,
            undo_log: UndoLog::new(),
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

    /// Get a player by ID
    pub fn get_player(&self, id: PlayerId) -> Result<&Player> {
        self.players
            .iter()
            .find(|p| p.id == id)
            .ok_or(crate::MtgError::EntityNotFound(id.as_u32()))
    }

    /// Get a mutable player by ID
    pub fn get_player_mut(&mut self, id: PlayerId) -> Result<&mut Player> {
        self.players
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or(crate::MtgError::EntityNotFound(id.as_u32()))
    }

    /// Get player by index (for stable turn order)
    pub fn get_player_by_idx(&self, idx: usize) -> Option<&Player> {
        self.players.get(idx)
    }

    /// Get mutable player by index
    pub fn get_player_by_idx_mut(&mut self, idx: usize) -> Option<&mut Player> {
        self.players.get_mut(idx)
    }

    /// Get the index of a player by ID
    pub fn get_player_idx(&self, id: PlayerId) -> Option<usize> {
        self.players.iter().position(|p| p.id == id)
    }

    /// Get the next player in turn order (for 2+ players)
    pub fn get_next_player_idx(&self, current_idx: usize) -> usize {
        (current_idx + 1) % self.players.len()
    }

    /// For 2-player games, get the other player's index
    pub fn get_other_player_idx(&self, player_idx: usize) -> Option<usize> {
        if self.players.len() == 2 {
            Some(1 - player_idx)
        } else {
            None
        }
    }

    /// For 2-player games, get the other player's ID
    pub fn get_other_player_id(&self, player_id: PlayerId) -> Option<PlayerId> {
        if self.players.len() == 2 {
            self.players
                .iter()
                .find(|p| p.id != player_id)
                .map(|p| p.id)
        } else {
            None
        }
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
                "Card {card_id} not found in source zone"
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

        // Log the action
        self.undo_log.log(crate::undo::GameAction::MoveCard {
            card_id,
            from_zone: from,
            to_zone: to,
            owner,
        });

        Ok(())
    }

    /// Draw a card for a player
    pub fn draw_card(&mut self, player_id: PlayerId) -> Result<Option<CardId>> {
        if let Some(zones) = self.get_player_zones_mut(player_id) {
            if let Some(card_id) = zones.library.draw_top() {
                zones.hand.add(card_id);

                // Log the card movement for undo
                self.undo_log.log(crate::undo::GameAction::MoveCard {
                    card_id,
                    from_zone: crate::zones::Zone::Library,
                    to_zone: crate::zones::Zone::Hand,
                    owner: player_id,
                });

                return Ok(Some(card_id));
            }
        }
        Ok(None)
    }

    /// Untap all permanents controlled by a player
    pub fn untap_all(&mut self, player_id: PlayerId) -> Result<()> {
        for card_id in self.battlefield.cards.iter() {
            if let Ok(card) = self.cards.get_mut(*card_id) {
                if card.controller == player_id && card.tapped {
                    card.untap();
                    // Log the untap action
                    self.undo_log.log(crate::undo::GameAction::TapCard {
                        card_id: *card_id,
                        tapped: false,
                    });
                }
            }
        }
        Ok(())
    }

    /// Clear temporary effects at end of turn (Cleanup step)
    /// This resets power/toughness bonuses from pump spells
    pub fn cleanup_temporary_effects(&mut self) {
        for card_id in self.battlefield.cards.iter() {
            if let Ok(card) = self.cards.get_mut(*card_id) {
                // Reset temporary bonuses (pump effects last until end of turn)
                card.power_bonus = 0;
                card.toughness_bonus = 0;
            }
        }
    }

    /// Advance the game to the next step
    pub fn advance_step(&mut self) -> Result<()> {
        let from_step = self.turn.current_step;

        // If entering cleanup step, clean up temporary effects
        if from_step == crate::game::Step::End
            && self.turn.current_step.next() == Some(crate::game::Step::Cleanup)
        {
            self.cleanup_temporary_effects();
        }

        if !self.turn.advance_step() {
            // End of turn, move to next player
            let from_player = self.turn.active_player;
            let next_player = self.get_next_player(self.turn.active_player)?;
            let old_turn_number = self.turn.turn_number;

            self.turn.next_turn(next_player);

            // Log the turn change
            self.undo_log.log(crate::undo::GameAction::ChangeTurn {
                from_player,
                to_player: next_player,
                turn_number: old_turn_number + 1,
            });

            // Reset per-turn state
            if let Ok(player) = self.get_player_mut(next_player) {
                player.reset_lands_played();
            }
        } else {
            // Log the step advance
            self.undo_log.log(crate::undo::GameAction::AdvanceStep {
                from_step,
                to_step: self.turn.current_step,
            });
        }
        Ok(())
    }

    /// Get the next player in turn order
    fn get_next_player(&self, current_player: PlayerId) -> Result<PlayerId> {
        let current_idx = self
            .get_player_idx(current_player)
            .ok_or(crate::MtgError::EntityNotFound(current_player.as_u32()))?;
        let next_idx = self.get_next_player_idx(current_idx);
        Ok(self.players[next_idx].id)
    }

    /// Check if the game is over
    pub fn is_game_over(&self) -> bool {
        self.players.iter().filter(|p| !p.has_lost).count() <= 1
    }

    /// Get the winner (if game is over)
    pub fn get_winner(&self) -> Option<PlayerId> {
        if !self.is_game_over() {
            return None;
        }
        self.players.iter().find(|p| !p.has_lost).map(|p| p.id)
    }

    /// Undo the most recent action
    ///
    /// Pops the last action from the undo log and reverts it.
    /// Returns Ok(true) if an action was undone, Ok(false) if log is empty.
    pub fn undo(&mut self) -> Result<bool> {
        if let Some(action) = self.undo_log.pop() {
            match action {
                crate::undo::GameAction::MoveCard {
                    card_id,
                    from_zone,
                    to_zone,
                    owner,
                } => {
                    // Move card back from to_zone to from_zone
                    // Don't log this action since it's a revert
                    let removed = match to_zone {
                        Zone::Battlefield => self.battlefield.remove(card_id),
                        Zone::Stack => self.stack.remove(card_id),
                        _ => {
                            if let Some(zones) = self.get_player_zones_mut(owner) {
                                if let Some(zone) = zones.get_zone_mut(to_zone) {
                                    zone.remove(card_id)
                                } else {
                                    eprintln!("UNDO BUG: Failed to get zone {:?} for undo", to_zone);
                                    false
                                }
                            } else {
                                eprintln!("UNDO BUG: Failed to get player zones for {:?}", owner);
                                false
                            }
                        }
                    };

                    if !removed {
                        // Find where the card actually is
                        let mut actual_zone = None;
                        if self.battlefield.contains(card_id) {
                            actual_zone = Some("Battlefield");
                        } else if self.stack.contains(card_id) {
                            actual_zone = Some("Stack");
                        } else if let Some(zones) = self.get_player_zones(owner) {
                            if zones.hand.contains(card_id) {
                                actual_zone = Some("Hand");
                            } else if zones.library.contains(card_id) {
                                actual_zone = Some("Library");
                            } else if zones.graveyard.contains(card_id) {
                                actual_zone = Some("Graveyard");
                            } else if zones.exile.contains(card_id) {
                                actual_zone = Some("Exile");
                            }
                        }
                        eprintln!("UNDO BUG: Card {} not found in to_zone {:?}, cannot undo move from {:?} → {:?}. Card is actually in: {:?}",
                                  card_id.as_u32(), to_zone, from_zone, to_zone, actual_zone);
                    } else {
                        match from_zone {
                            Zone::Battlefield => self.battlefield.add(card_id),
                            Zone::Stack => self.stack.add(card_id),
                            _ => {
                                if let Some(zones) = self.get_player_zones_mut(owner) {
                                    if let Some(zone) = zones.get_zone_mut(from_zone) {
                                        zone.add(card_id);
                                    }
                                }
                            }
                        }
                    }
                }
                crate::undo::GameAction::TapCard { card_id, tapped } => {
                    // Reverse the tap state
                    if let Ok(card) = self.cards.get_mut(card_id) {
                        if tapped {
                            card.untap();
                        } else {
                            card.tap();
                        }
                    }
                }
                crate::undo::GameAction::ModifyLife { player_id, delta } => {
                    // Apply the negative of the delta
                    if let Ok(player) = self.get_player_mut(player_id) {
                        if delta > 0 {
                            player.lose_life(delta);
                        } else {
                            player.gain_life(-delta);
                        }
                        // Recheck has_lost status
                        if player.life > 0 {
                            player.has_lost = false;
                        }
                    }
                }
                crate::undo::GameAction::AddMana { player_id, color } => {
                    // Remove the mana that was added
                    if let Ok(player) = self.get_player_mut(player_id) {
                        match color {
                            crate::core::Color::White => {
                                if player.mana_pool.white > 0 {
                                    player.mana_pool.white -= 1;
                                }
                            }
                            crate::core::Color::Blue => {
                                if player.mana_pool.blue > 0 {
                                    player.mana_pool.blue -= 1;
                                }
                            }
                            crate::core::Color::Black => {
                                if player.mana_pool.black > 0 {
                                    player.mana_pool.black -= 1;
                                }
                            }
                            crate::core::Color::Red => {
                                if player.mana_pool.red > 0 {
                                    player.mana_pool.red -= 1;
                                }
                            }
                            crate::core::Color::Green => {
                                if player.mana_pool.green > 0 {
                                    player.mana_pool.green -= 1;
                                }
                            }
                            crate::core::Color::Colorless => {
                                if player.mana_pool.colorless > 0 {
                                    player.mana_pool.colorless -= 1;
                                }
                            }
                        }
                    }
                }
                crate::undo::GameAction::EmptyManaPool {
                    player_id,
                    prev_white,
                    prev_blue,
                    prev_black,
                    prev_red,
                    prev_green,
                    prev_colorless,
                } => {
                    // Restore previous mana pool state
                    if let Ok(player) = self.get_player_mut(player_id) {
                        player.mana_pool.white = prev_white;
                        player.mana_pool.blue = prev_blue;
                        player.mana_pool.black = prev_black;
                        player.mana_pool.red = prev_red;
                        player.mana_pool.green = prev_green;
                        player.mana_pool.colorless = prev_colorless;
                    }
                }
                crate::undo::GameAction::AddCounter { .. } => {
                    // TODO: Implement counter undo
                }
                crate::undo::GameAction::RemoveCounter { .. } => {
                    // TODO: Implement counter undo
                }
                crate::undo::GameAction::AdvanceStep {
                    from_step,
                    to_step: _,
                } => {
                    // Revert to previous step
                    self.turn.current_step = from_step;
                }
                crate::undo::GameAction::ChangeTurn {
                    from_player,
                    to_player: _,
                    turn_number,
                } => {
                    // Revert to previous turn
                    self.turn.active_player = from_player;
                    self.turn.turn_number = turn_number - 1;
                    // Note: We don't reset lands_played here as that state
                    // should be managed by separate actions if needed
                }
                crate::undo::GameAction::PumpCreature {
                    card_id,
                    power_delta,
                    toughness_delta,
                } => {
                    // Reverse the pump effect
                    if let Ok(card) = self.cards.get_mut(card_id) {
                        card.power_bonus -= power_delta;
                        card.toughness_bonus -= toughness_delta;
                    }
                }
                crate::undo::GameAction::ChoicePoint { .. } => {
                    // Choice points don't need to be undone
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
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
        let p1_id = game.players.first().unwrap().id; // Copy the ID
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
        let p1_id = game.players.first().unwrap().id; // Copy the ID
        if let Ok(player) = game.get_player_mut(p1_id) {
            player.lose_life(20);
        }

        assert!(game.is_game_over());
        let winner = game.get_winner().unwrap();
        assert_ne!(winner, p1_id);
    }

    #[test]
    fn test_undo_log_integration() {
        use crate::core::CardType;

        let mut game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let p1_id = game.players.first().unwrap().id;

        assert_eq!(game.undo_log.len(), 0);

        // Create and play a land
        let card_id = game.next_card_id();
        let mut card = Card::new(card_id, "Mountain", p1_id);
        card.types.push(CardType::Land);
        game.cards.insert(card_id, card);

        if let Some(zones) = game.get_player_zones_mut(p1_id) {
            zones.hand.add(card_id);
        }

        // Play the land - should log MoveCard
        game.play_land(p1_id, card_id).unwrap();
        assert_eq!(game.undo_log.len(), 1);
        matches!(
            game.undo_log.peek().unwrap(),
            crate::undo::GameAction::MoveCard { .. }
        );

        // Tap for mana - should log TapCard and AddMana
        game.tap_for_mana(p1_id, card_id).unwrap();
        assert_eq!(game.undo_log.len(), 3); // MoveCard, TapCard, AddMana

        // Untap all - should log TapCard for untap
        game.untap_all(p1_id).unwrap();
        assert_eq!(game.undo_log.len(), 4); // + TapCard (untapped)

        // Verify all actions are logged
        let actions = game.undo_log.actions();
        assert!(matches!(
            actions[0],
            crate::undo::GameAction::MoveCard { .. }
        ));
        assert!(matches!(
            actions[1],
            crate::undo::GameAction::TapCard { tapped: true, .. }
        ));
        assert!(matches!(
            actions[2],
            crate::undo::GameAction::AddMana { .. }
        ));
        assert!(matches!(
            actions[3],
            crate::undo::GameAction::TapCard { tapped: false, .. }
        ));
    }
}
