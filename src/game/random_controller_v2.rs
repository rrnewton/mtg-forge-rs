//! Random AI controller implementing the new PlayerController interface
//!
//! This is the v2 implementation using specific callback methods instead of
//! generic action choices. Makes random choices from available options.

use crate::core::{CardId, PlayerId};
use crate::game::controller::GameStateView;
use crate::game::controller_v2::PlayerController;
use rand::seq::SliceRandom;
use rand::Rng;
use smallvec::SmallVec;

/// A controller that makes random choices using the new callback interface
pub struct RandomControllerV2 {
    player_id: PlayerId,
    rng: Box<dyn rand::RngCore>,
}

impl RandomControllerV2 {
    /// Create a new random controller with default RNG
    pub fn new(player_id: PlayerId) -> Self {
        RandomControllerV2 {
            player_id,
            rng: Box::new(rand::thread_rng()),
        }
    }

    /// Create a random controller with a seeded RNG (for deterministic testing)
    pub fn with_seed(player_id: PlayerId, seed: u64) -> Self {
        use rand::SeedableRng;
        RandomControllerV2 {
            player_id,
            rng: Box::new(rand::rngs::StdRng::seed_from_u64(seed)),
        }
    }
}

impl PlayerController for RandomControllerV2 {
    fn player_id(&self) -> PlayerId {
        self.player_id
    }

    fn choose_land_to_play(
        &mut self,
        _view: &GameStateView,
        lands_in_hand: &[CardId],
    ) -> Option<CardId> {
        if lands_in_hand.is_empty() {
            None
        } else {
            // Randomly choose a land to play
            let index = self.rng.gen_range(0..lands_in_hand.len());
            Some(lands_in_hand[index])
        }
    }

    fn choose_spell_to_cast(
        &mut self,
        _view: &GameStateView,
        castable_spells: &[CardId],
    ) -> Option<(CardId, SmallVec<[CardId; 4]>)> {
        if castable_spells.is_empty() {
            None
        } else {
            // Randomly choose a spell to cast
            let index = self.rng.gen_range(0..castable_spells.len());
            let spell_id = castable_spells[index];

            // For now, return empty targets - targeting will be improved later
            let targets = SmallVec::new();

            Some((spell_id, targets))
        }
    }

    fn choose_card_to_tap_for_mana(
        &mut self,
        _view: &GameStateView,
        tappable_cards: &[CardId],
    ) -> Option<CardId> {
        if tappable_cards.is_empty() {
            None
        } else {
            // Randomly choose a card to tap for mana
            let index = self.rng.gen_range(0..tappable_cards.len());
            Some(tappable_cards[index])
        }
    }

    fn choose_attackers(
        &mut self,
        _view: &GameStateView,
        available_creatures: &[CardId],
    ) -> SmallVec<[CardId; 8]> {
        // Randomly decide whether each creature attacks
        let mut attackers = SmallVec::new();

        for &creature_id in available_creatures {
            // 50% chance each creature attacks
            if self.rng.gen_bool(0.5) {
                attackers.push(creature_id);
            }
        }

        attackers
    }

    fn choose_blockers(
        &mut self,
        _view: &GameStateView,
        available_blockers: &[CardId],
        attackers: &[CardId],
    ) -> SmallVec<[(CardId, CardId); 8]> {
        // Randomly assign blockers to attackers
        let mut blocks = SmallVec::new();

        if attackers.is_empty() {
            return blocks;
        }

        for &blocker_id in available_blockers {
            // 50% chance each creature blocks
            if self.rng.gen_bool(0.5) {
                // Pick a random attacker to block
                let attacker_idx = self.rng.gen_range(0..attackers.len());
                blocks.push((blocker_id, attackers[attacker_idx]));
            }
        }

        blocks
    }

    fn choose_cards_to_discard(
        &mut self,
        _view: &GameStateView,
        hand: &[CardId],
        count: usize,
    ) -> SmallVec<[CardId; 7]> {
        // Randomly choose cards to discard from hand
        let mut hand_vec: Vec<CardId> = hand.to_vec();
        hand_vec.shuffle(&mut self.rng);

        hand_vec
            .iter()
            .take(count.min(hand.len()))
            .copied()
            .collect()
    }

    fn wants_to_pass_priority(&mut self, _view: &GameStateView) -> bool {
        // Random controller passes priority with 70% probability
        // This prevents infinite loops while still allowing some actions
        self.rng.gen_bool(0.7)
    }

    fn on_priority_passed(&mut self, _view: &GameStateView) {
        // Random AI doesn't need to react to priority passes
    }

    fn on_game_end(&mut self, _view: &GameStateView, _won: bool) {
        // Could log game result here for statistics
        // Disabled for quiet operation during benchmarks and batch runs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EntityId;
    use crate::game::GameState;

    #[test]
    fn test_random_controller_v2_creation() {
        let player_id = EntityId::new(1);
        let controller = RandomControllerV2::new(player_id);
        assert_eq!(controller.player_id(), player_id);
    }

    #[test]
    fn test_seeded_controller_v2() {
        let player_id = EntityId::new(1);
        let controller = RandomControllerV2::with_seed(player_id, 42);
        assert_eq!(controller.player_id(), player_id);
    }

    #[test]
    fn test_choose_land_to_play_empty() {
        let player_id = EntityId::new(1);
        let mut controller = RandomControllerV2::with_seed(player_id, 42);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        // With no lands, should return None
        let land = controller.choose_land_to_play(&view, &[]);
        assert_eq!(land, None);
    }

    #[test]
    fn test_choose_land_to_play() {
        let player_id = EntityId::new(1);
        let mut controller = RandomControllerV2::with_seed(player_id, 42);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let lands = vec![EntityId::new(10), EntityId::new(11), EntityId::new(12)];
        let chosen = controller.choose_land_to_play(&view, &lands);

        // Should choose one of the available lands
        assert!(chosen.is_some());
        assert!(lands.contains(&chosen.unwrap()));
    }

    #[test]
    fn test_seeded_determinism_v2() {
        let player_id = EntityId::new(1);
        let mut controller1 = RandomControllerV2::with_seed(player_id, 42);
        let mut controller2 = RandomControllerV2::with_seed(player_id, 42);

        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let lands = vec![EntityId::new(10), EntityId::new(11), EntityId::new(12)];

        // Same seed should produce same choices
        let choice1 = controller1.choose_land_to_play(&view, &lands);
        let choice2 = controller2.choose_land_to_play(&view, &lands);

        assert_eq!(choice1, choice2);
    }

    #[test]
    fn test_choose_attackers() {
        let player_id = EntityId::new(1);
        let mut controller = RandomControllerV2::with_seed(player_id, 42);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let creatures = vec![EntityId::new(20), EntityId::new(21), EntityId::new(22)];
        let attackers = controller.choose_attackers(&view, &creatures);

        // Should return a SmallVec (possibly empty)
        // All attackers should be from the available creatures
        for attacker in attackers.iter() {
            assert!(creatures.contains(attacker));
        }
    }

    #[test]
    fn test_choose_cards_to_discard() {
        let player_id = EntityId::new(1);
        let mut controller = RandomControllerV2::with_seed(player_id, 42);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let hand = vec![
            EntityId::new(30),
            EntityId::new(31),
            EntityId::new(32),
            EntityId::new(33),
        ];

        let discards = controller.choose_cards_to_discard(&view, &hand, 2);

        // Should discard exactly 2 cards
        assert_eq!(discards.len(), 2);

        // All discarded cards should be from hand
        for card in discards.iter() {
            assert!(hand.contains(card));
        }
    }
}
