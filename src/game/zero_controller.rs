//! Zero controller for testing and automation
//!
//! This controller always chooses the first available action (or passes if no actions).
//! It's useful for automated testing and ensuring games can complete without user input.
//! Equivalent to the Java Forge "--p1=zero" option.

use crate::core::{CardId, PlayerId};
use crate::game::controller::{GameStateView, PlayerAction, PlayerController};

/// A controller that always chooses the first available action (index 0)
///
/// This is useful for:
/// - Automated testing
/// - Running games without interaction
/// - Benchmarking the game engine
/// - Ensuring games can complete deterministically
pub struct ZeroController {
    player_id: PlayerId,
}

impl ZeroController {
    /// Create a new zero controller
    pub fn new(player_id: PlayerId) -> Self {
        ZeroController { player_id }
    }
}

impl PlayerController for ZeroController {
    fn player_id(&self) -> PlayerId {
        self.player_id
    }

    fn choose_action(
        &mut self,
        _view: &GameStateView,
        available_actions: &[PlayerAction],
    ) -> Option<PlayerAction> {
        // Filter out PassPriority to see if there are any meaningful actions
        let meaningful_actions: Vec<&PlayerAction> = available_actions
            .iter()
            .filter(|a| !matches!(a, PlayerAction::PassPriority | PlayerAction::TapForMana(_)))
            .collect();

        // If there are meaningful actions, choose the first one
        if let Some(action) = meaningful_actions.first() {
            return Some((*action).clone());
        }

        // Otherwise pass priority (return None)
        None
    }

    fn choose_cards_to_discard(&mut self, view: &GameStateView, count: usize) -> Vec<CardId> {
        // Zero controller discards the first N cards in hand
        let hand = view.player_hand(self.player_id);
        hand.iter().take(count).copied().collect()
    }

    fn on_priority_passed(&mut self, _view: &GameStateView) {
        // Zero controller doesn't need to log
    }

    fn on_game_end(&mut self, _view: &GameStateView, _won: bool) {
        // Zero controller doesn't need to log
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EntityId;
    use crate::game::GameState;

    #[test]
    fn test_zero_controller_chooses_first() {
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let player_id = *game.players.iter().next().unwrap().0;

        let mut controller = ZeroController::new(player_id);
        let view = GameStateView::new(&game, player_id);

        let card_id1 = EntityId::new(10);
        let card_id2 = EntityId::new(11);
        let actions = vec![
            PlayerAction::PlayLand(card_id1),
            PlayerAction::TapForMana(card_id2),
            PlayerAction::PassPriority,
        ];

        // Should always choose first action
        let choice = controller.choose_action(&view, &actions);
        assert_eq!(choice, Some(PlayerAction::PlayLand(card_id1)));
    }

    #[test]
    fn test_zero_controller_empty_actions() {
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let player_id = *game.players.iter().next().unwrap().0;

        let mut controller = ZeroController::new(player_id);
        let view = GameStateView::new(&game, player_id);

        // No actions available
        let choice = controller.choose_action(&view, &[]);
        assert_eq!(choice, None);
    }
}
