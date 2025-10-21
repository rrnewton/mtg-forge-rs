//! Scripted player controller for testing and examples
//!
//! This controller follows a predetermined script of actions,
//! useful for examples and deterministic testing.

use crate::core::{CardId, PlayerId};
use crate::game::controller::{GameStateView, PlayerAction, PlayerController};

/// A controller that follows a predetermined sequence of actions
pub struct ScriptedController {
    player_id: PlayerId,
    actions: Vec<PlayerAction>,
    current_step: usize,
}

impl ScriptedController {
    /// Create a new scripted controller with a sequence of actions
    pub fn new(player_id: PlayerId, actions: Vec<PlayerAction>) -> Self {
        ScriptedController {
            player_id,
            actions,
            current_step: 0,
        }
    }
}

impl PlayerController for ScriptedController {
    fn player_id(&self) -> PlayerId {
        self.player_id
    }

    fn choose_action(
        &mut self,
        _view: &GameStateView,
        _available_actions: &[PlayerAction],
    ) -> Option<PlayerAction> {
        if self.current_step < self.actions.len() {
            let action = self.actions[self.current_step].clone();
            self.current_step += 1;
            Some(action)
        } else {
            // No more scripted actions, pass priority
            None
        }
    }

    fn choose_cards_to_discard(&mut self, view: &GameStateView, count: usize) -> Vec<CardId> {
        // Scripted controller discards the first N cards in hand
        let hand = view.player_hand(self.player_id);
        hand.iter().take(count).copied().collect()
    }

    fn on_priority_passed(&mut self, _view: &GameStateView) {
        // Could log here for debugging
    }

    fn on_game_end(&mut self, _view: &GameStateView, _won: bool) {
        // Could log final game state here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EntityId;
    use crate::game::GameState;

    #[test]
    fn test_scripted_controller() {
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let player_id = *game.players.iter().next().unwrap().0;

        let card_id = EntityId::new(10);
        let actions = vec![
            PlayerAction::PlayLand(card_id),
            PlayerAction::TapForMana(card_id),
            PlayerAction::PassPriority,
        ];

        let mut controller = ScriptedController::new(player_id, actions);
        let view = GameStateView::new(&game, player_id);

        // First action
        let action1 = controller.choose_action(&view, &[]);
        assert_eq!(action1, Some(PlayerAction::PlayLand(card_id)));

        // Second action
        let action2 = controller.choose_action(&view, &[]);
        assert_eq!(action2, Some(PlayerAction::TapForMana(card_id)));

        // Third action
        let action3 = controller.choose_action(&view, &[]);
        assert_eq!(action3, Some(PlayerAction::PassPriority));

        // No more actions
        let action4 = controller.choose_action(&view, &[]);
        assert_eq!(action4, None);
    }
}
