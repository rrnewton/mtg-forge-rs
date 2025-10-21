//! Random AI controller for testing and baseline gameplay
//!
//! Makes random choices from available actions.
//! Serves as a baseline for more sophisticated AI.

use crate::core::{CardId, PlayerId};
use crate::game::controller::{GameStateView, PlayerAction, PlayerController};
use rand::seq::SliceRandom;
use rand::Rng;

/// A controller that makes random choices
pub struct RandomController {
    player_id: PlayerId,
    rng: Box<dyn rand::RngCore>,
}

impl RandomController {
    /// Create a new random controller with default RNG
    pub fn new(player_id: PlayerId) -> Self {
        RandomController {
            player_id,
            rng: Box::new(rand::thread_rng()),
        }
    }

    /// Create a random controller with a seeded RNG (for deterministic testing)
    pub fn with_seed(player_id: PlayerId, seed: u64) -> Self {
        use rand::SeedableRng;
        RandomController {
            player_id,
            rng: Box::new(rand::rngs::StdRng::seed_from_u64(seed)),
        }
    }
}

impl PlayerController for RandomController {
    fn player_id(&self) -> PlayerId {
        self.player_id
    }

    fn choose_action(
        &mut self,
        _view: &GameStateView,
        available_actions: &[PlayerAction],
    ) -> Option<PlayerAction> {
        if available_actions.is_empty() {
            // No actions available, pass priority
            None
        } else {
            // Filter out PassPriority to see what actual actions are available
            let non_pass_actions: Vec<&PlayerAction> = available_actions
                .iter()
                .filter(|a| !matches!(a, PlayerAction::PassPriority))
                .collect();

            // If only PassPriority is available, pass
            if non_pass_actions.is_empty() {
                return None;
            }

            // Strategy: Prefer meaningful actions over just tapping for mana
            // If we have spells to cast or lands to play, prioritize those
            let high_priority_actions: Vec<&PlayerAction> = non_pass_actions
                .iter()
                .filter(|a| {
                    matches!(
                        a,
                        PlayerAction::CastSpell { .. } | PlayerAction::PlayLand(_)
                    )
                })
                .copied()
                .collect();

            // 70% chance to pass if only mana tapping is available
            // This prevents infinite mana-tapping loops
            if high_priority_actions.is_empty() && self.rng.gen_bool(0.7) {
                return None;
            }

            // Choose from high priority actions if available, otherwise from all actions
            let action_pool = if !high_priority_actions.is_empty() {
                high_priority_actions
            } else {
                non_pass_actions
            };

            let index = self.rng.gen_range(0..action_pool.len());
            Some(action_pool[index].clone())
        }
    }

    fn choose_cards_to_discard(&mut self, view: &GameStateView, count: usize) -> Vec<CardId> {
        // Random controller discards random cards from hand
        let mut hand: Vec<CardId> = view.player_hand(self.player_id).to_vec();
        hand.shuffle(&mut self.rng);
        hand.iter().take(count).copied().collect()
    }

    fn on_priority_passed(&mut self, _view: &GameStateView) {
        // Random AI doesn't need to react to priority passes
    }

    fn on_game_end(&mut self, _view: &GameStateView, _won: bool) {
        // Could log game result here for statistics
        // Disabled for quiet operation during benchmarks and batch runs
        // let life = view.life();
        // if won {
        //     println!("Random AI wins with {life} life!");
        // } else {
        //     println!("Random AI loses with {life} life.");
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EntityId;
    use crate::game::GameState;

    #[test]
    fn test_random_controller_creation() {
        let player_id = EntityId::new(1);
        let controller = RandomController::new(player_id);
        assert_eq!(controller.player_id(), player_id);
    }

    #[test]
    fn test_seeded_controller() {
        let player_id = EntityId::new(1);
        let controller = RandomController::with_seed(player_id, 42);
        assert_eq!(controller.player_id(), player_id);
    }

    #[test]
    fn test_choose_from_empty_actions() {
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let player_id = *game.players.iter().next().unwrap().0;

        let mut controller = RandomController::with_seed(player_id, 42);
        let view = GameStateView::new(&game, player_id);

        // With no available actions, should return None
        let action = controller.choose_action(&view, &[]);
        assert_eq!(action, None);
    }

    #[test]
    fn test_choose_from_actions() {
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let player_id = *game.players.iter().next().unwrap().0;

        let mut controller = RandomController::with_seed(player_id, 42);
        let view = GameStateView::new(&game, player_id);

        let card_id = EntityId::new(10);
        let actions = vec![
            PlayerAction::PlayLand(card_id),
            PlayerAction::TapForMana(card_id),
            PlayerAction::PassPriority,
        ];

        // Should choose one of the available actions
        let action = controller.choose_action(&view, &actions);
        assert!(action.is_some());
        assert!(actions.contains(&action.unwrap()));
    }

    #[test]
    fn test_seeded_determinism() {
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let player_id = *game.players.iter().next().unwrap().0;

        let mut controller1 = RandomController::with_seed(player_id, 42);
        let mut controller2 = RandomController::with_seed(player_id, 42);

        let view = GameStateView::new(&game, player_id);
        let card_id = EntityId::new(10);
        let actions = vec![
            PlayerAction::PlayLand(card_id),
            PlayerAction::TapForMana(card_id),
            PlayerAction::PassPriority,
        ];

        // Same seed should produce same choices
        let action1 = controller1.choose_action(&view, &actions);
        let action2 = controller2.choose_action(&view, &actions);

        assert_eq!(action1, action2);
    }
}
