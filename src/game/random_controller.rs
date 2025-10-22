//! Random AI controller implementing the new PlayerController interface
//!
//! This implementation uses specific callback methods instead of
//! generic action choices. Makes random choices from available options.

use crate::core::{CardId, ManaCost, PlayerId, SpellAbility};
use crate::game::controller::GameStateView;
use crate::game::controller::PlayerController;
use rand::seq::SliceRandom;
use rand::Rng;
use smallvec::SmallVec;

/// A controller that makes random choices using the new callback interface
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

    fn choose_spell_ability_to_play(
        &mut self,
        _view: &GameStateView,
        available: &[SpellAbility],
    ) -> Option<SpellAbility> {
        if available.is_empty() {
            // No available actions - pass priority
            None
        } else {
            // Random controller passes priority with 30% probability
            // This allows actions to be taken most of the time while still preventing infinite loops
            if self.rng.gen_bool(0.3) {
                return None;
            }

            // Randomly choose one of the available spell abilities
            let index = self.rng.gen_range(0..available.len());
            Some(available[index].clone())
        }
    }

    fn choose_targets(
        &mut self,
        _view: &GameStateView,
        _spell: CardId,
        valid_targets: &[CardId],
    ) -> SmallVec<[CardId; 4]> {
        // For now, just pick a random target if any are available
        // TODO: Improve targeting logic based on spell requirements
        if valid_targets.is_empty() {
            SmallVec::new()
        } else {
            let index = self.rng.gen_range(0..valid_targets.len());
            let mut targets = SmallVec::new();
            targets.push(valid_targets[index]);
            targets
        }
    }

    fn choose_mana_sources_to_pay(
        &mut self,
        _view: &GameStateView,
        cost: &ManaCost,
        available_sources: &[CardId],
    ) -> SmallVec<[CardId; 8]> {
        // Simple greedy approach: tap sources until we have enough mana
        // TODO: Improve to consider mana colors and optimization
        let mut sources = SmallVec::new();
        let needed = cost.cmc() as usize;

        // Shuffle to randomize which sources we choose
        let mut shuffled: Vec<CardId> = available_sources.to_vec();
        shuffled.shuffle(&mut self.rng);

        for &source_id in shuffled.iter().take(needed) {
            sources.push(source_id);
        }

        sources
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
    fn test_choose_spell_ability_empty() {
        let player_id = EntityId::new(1);
        let mut controller = RandomController::with_seed(player_id, 42);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        // With no available abilities, should return None
        let choice = controller.choose_spell_ability_to_play(&view, &[]);
        assert_eq!(choice, None);
    }

    #[test]
    fn test_choose_spell_ability() {
        let player_id = EntityId::new(1);
        let mut controller = RandomController::with_seed(player_id, 42);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let abilities = vec![
            SpellAbility::PlayLand {
                card_id: EntityId::new(10),
            },
            SpellAbility::CastSpell {
                card_id: EntityId::new(11),
            },
        ];

        // May choose an ability or pass (due to 30% pass probability)
        // Try multiple times to ensure it makes choices sometimes
        let mut found_choice = false;
        for _ in 0..20 {
            let choice = controller.choose_spell_ability_to_play(&view, &abilities);
            if choice.is_some() {
                found_choice = true;
                // The choice should be one of the available abilities
                assert!(abilities.contains(&choice.unwrap()));
            }
        }
        // With 30% pass rate, over 20 tries we should see at least one choice
        assert!(found_choice);
    }

    #[test]
    fn test_choose_targets() {
        let player_id = EntityId::new(1);
        let mut controller = RandomController::with_seed(player_id, 42);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let spell_id = EntityId::new(100);
        let valid_targets = vec![EntityId::new(20), EntityId::new(21), EntityId::new(22)];
        let targets = controller.choose_targets(&view, spell_id, &valid_targets);

        // Should choose exactly one target
        assert_eq!(targets.len(), 1);
        // Target should be from the valid list
        assert!(valid_targets.contains(&targets[0]));
    }

    #[test]
    fn test_choose_mana_sources() {
        let player_id = EntityId::new(1);
        let mut controller = RandomController::with_seed(player_id, 42);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let cost = ManaCost::from_string("2RR"); // CMC = 4
        let available = vec![
            EntityId::new(10),
            EntityId::new(11),
            EntityId::new(12),
            EntityId::new(13),
            EntityId::new(14),
        ];

        let sources = controller.choose_mana_sources_to_pay(&view, &cost, &available);

        // Should choose exactly 4 sources (equal to CMC)
        assert_eq!(sources.len(), 4);
        // All sources should be from the available list
        for source in sources.iter() {
            assert!(available.contains(source));
        }
    }

    #[test]
    fn test_choose_attackers() {
        let player_id = EntityId::new(1);
        let mut controller = RandomController::with_seed(player_id, 42);
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
        let mut controller = RandomController::with_seed(player_id, 42);
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
