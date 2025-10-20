//! Player controller trait and game state view
//!
//! This module defines the interface between the game engine and player
//! controllers (AI or human). The game engine calls the controller when
//! decisions need to be made, and the controller inspects a read-only
//! view of the game state to make choices.

use crate::core::{CardId, PlayerId};
use crate::game::GameState;
use crate::zones::Zone;

/// Available actions a player can take
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerAction {
    /// Play a land card from hand
    PlayLand(CardId),

    /// Cast a spell from hand
    /// For now, targets are simplified - we'll expand this later
    CastSpell {
        card_id: CardId,
        targets: Vec<CardId>,
    },

    /// Activate a mana ability (tap land for mana)
    TapForMana(CardId),

    /// Pass priority (do nothing)
    PassPriority,
}

/// Read-only view of game state for controllers
///
/// This provides access to game information without allowing mutation.
/// Controllers should only inspect this view to make decisions.
pub struct GameStateView<'a> {
    game: &'a GameState,
    player_id: PlayerId,
}

impl<'a> GameStateView<'a> {
    /// Create a new view of the game state from a player's perspective
    pub fn new(game: &'a GameState, player_id: PlayerId) -> Self {
        GameStateView { game, player_id }
    }

    /// Get the player ID this view is for
    pub fn player_id(&self) -> PlayerId {
        self.player_id
    }

    /// Get cards in this player's hand
    pub fn hand(&self) -> &[CardId] {
        self.game
            .get_player_zones(self.player_id)
            .map(|zones| zones.hand.cards.as_slice())
            .unwrap_or(&[])
    }

    /// Get cards on the battlefield
    pub fn battlefield(&self) -> &[CardId] {
        &self.game.battlefield.cards
    }

    /// Check if a card is in a specific zone
    pub fn is_card_in_zone(&self, card_id: CardId, zone: Zone) -> bool {
        match zone {
            Zone::Hand => self
                .game
                .get_player_zones(self.player_id)
                .map(|z| z.hand.contains(card_id))
                .unwrap_or(false),
            Zone::Battlefield => self.game.battlefield.contains(card_id),
            Zone::Graveyard => self
                .game
                .get_player_zones(self.player_id)
                .map(|z| z.graveyard.contains(card_id))
                .unwrap_or(false),
            Zone::Library => self
                .game
                .get_player_zones(self.player_id)
                .map(|z| z.library.contains(card_id))
                .unwrap_or(false),
            Zone::Stack => self.game.stack.contains(card_id),
            Zone::Exile => self
                .game
                .get_player_zones(self.player_id)
                .map(|z| z.exile.contains(card_id))
                .unwrap_or(false),
            Zone::Command => false, // Command zone not yet implemented in PlayerZones
        }
    }

    /// Get a card's name
    pub fn card_name(&self, card_id: CardId) -> Option<String> {
        self.game
            .cards
            .get(card_id)
            .ok()
            .map(|c| c.name.to_string())
    }

    /// Check if a card is a land
    pub fn is_land(&self, card_id: CardId) -> bool {
        self.game
            .cards
            .get(card_id)
            .map(|c| c.is_land())
            .unwrap_or(false)
    }

    /// Check if a card is tapped
    pub fn is_tapped(&self, card_id: CardId) -> bool {
        self.game
            .cards
            .get(card_id)
            .map(|c| c.tapped)
            .unwrap_or(false)
    }

    /// Get player's current life total
    pub fn life(&self) -> i32 {
        self.game
            .players
            .get(self.player_id)
            .map(|p| p.life)
            .unwrap_or(0)
    }

    /// Get player's mana pool
    pub fn available_mana(&self) -> (u8, u8, u8, u8, u8, u8) {
        // Returns (white, blue, black, red, green, colorless)
        self.game
            .players
            .get(self.player_id)
            .map(|p| {
                (
                    p.mana_pool.white,
                    p.mana_pool.blue,
                    p.mana_pool.black,
                    p.mana_pool.red,
                    p.mana_pool.green,
                    p.mana_pool.colorless,
                )
            })
            .unwrap_or((0, 0, 0, 0, 0, 0))
    }

    /// Check if player can play lands this turn
    pub fn can_play_land(&self) -> bool {
        self.game
            .players
            .get(self.player_id)
            .map(|p| p.can_play_land())
            .unwrap_or(false)
    }
}

/// Player controller trait
///
/// Implement this trait to create AI players or connect to UI.
/// The game engine will call these methods when decisions need to be made.
pub trait PlayerController {
    /// Get the player ID this controller is responsible for
    fn player_id(&self) -> PlayerId;

    /// Choose an action from available options
    ///
    /// The game engine provides a view of the game state and a list of
    /// available actions. The controller should return one action, or
    /// None to pass priority.
    fn choose_action(
        &mut self,
        view: &GameStateView,
        available_actions: &[PlayerAction],
    ) -> Option<PlayerAction>;

    /// Called when priority passes (for logging/debugging)
    fn on_priority_passed(&mut self, _view: &GameStateView) {}

    /// Called when the game ends (for cleanup/logging)
    fn on_game_end(&mut self, _view: &GameStateView, _won: bool) {}
}
