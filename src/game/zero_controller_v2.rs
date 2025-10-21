//! Zero controller implementing the new PlayerController interface
//!
//! This controller uses simple deterministic heuristics:
//! - Plays the first land available
//! - Casts the first spell available
//! - Doesn't proactively tap for mana
//! - Attacks with all creatures
//! - Blocks each attacker with one blocker
//! - Discards the first N cards from hand

use crate::core::{CardId, PlayerId};
use crate::game::controller::GameStateView;
use crate::game::controller_v2::PlayerController;
use smallvec::SmallVec;

/// A controller that uses simple "first choice" heuristics
///
/// This is useful for:
/// - Automated testing
/// - Running games without interaction
/// - Benchmarking the game engine
/// - Ensuring games can complete deterministically
pub struct ZeroControllerV2 {
    player_id: PlayerId,
}

impl ZeroControllerV2 {
    /// Create a new zero controller
    pub fn new(player_id: PlayerId) -> Self {
        ZeroControllerV2 { player_id }
    }
}

impl PlayerController for ZeroControllerV2 {
    fn player_id(&self) -> PlayerId {
        self.player_id
    }

    fn choose_land_to_play(
        &mut self,
        _view: &GameStateView,
        lands_in_hand: &[CardId],
    ) -> Option<CardId> {
        // Play the first land available
        lands_in_hand.first().copied()
    }

    fn choose_spell_to_cast(
        &mut self,
        _view: &GameStateView,
        castable_spells: &[CardId],
    ) -> Option<(CardId, SmallVec<[CardId; 4]>)> {
        // Cast the first spell available with no targets
        castable_spells.first().map(|&spell_id| {
            let targets = SmallVec::new();
            (spell_id, targets)
        })
    }

    fn choose_card_to_tap_for_mana(
        &mut self,
        _view: &GameStateView,
        _tappable_cards: &[CardId],
    ) -> Option<CardId> {
        // Zero controller doesn't proactively tap for mana
        // (mana will be tapped automatically when needed by the game engine)
        None
    }

    fn choose_attackers(
        &mut self,
        _view: &GameStateView,
        available_creatures: &[CardId],
    ) -> SmallVec<[CardId; 8]> {
        // Attack with all available creatures
        available_creatures.iter().copied().collect()
    }

    fn choose_blockers(
        &mut self,
        _view: &GameStateView,
        available_blockers: &[CardId],
        attackers: &[CardId],
    ) -> SmallVec<[(CardId, CardId); 8]> {
        // Block each attacker with one blocker (if available)
        let mut blocks = SmallVec::new();

        for (i, &attacker_id) in attackers.iter().enumerate() {
            if let Some(&blocker_id) = available_blockers.get(i) {
                blocks.push((blocker_id, attacker_id));
            } else {
                // No more blockers available
                break;
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
        // Discard the first N cards from hand
        hand.iter().take(count.min(hand.len())).copied().collect()
    }

    fn wants_to_pass_priority(&mut self, _view: &GameStateView) -> bool {
        // Zero controller only acts when specifically asked to choose an action
        // It always passes priority (the game loop will call specific methods when needed)
        true
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
    fn test_zero_controller_v2_creation() {
        let player_id = EntityId::new(1);
        let controller = ZeroControllerV2::new(player_id);
        assert_eq!(controller.player_id(), player_id);
    }

    #[test]
    fn test_choose_land_to_play() {
        let player_id = EntityId::new(1);
        let mut controller = ZeroControllerV2::new(player_id);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let lands = vec![EntityId::new(10), EntityId::new(11), EntityId::new(12)];
        let chosen = controller.choose_land_to_play(&view, &lands);

        // Should choose the first land
        assert_eq!(chosen, Some(EntityId::new(10)));
    }

    #[test]
    fn test_choose_land_empty() {
        let player_id = EntityId::new(1);
        let mut controller = ZeroControllerV2::new(player_id);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let chosen = controller.choose_land_to_play(&view, &[]);

        // No lands available
        assert_eq!(chosen, None);
    }

    #[test]
    fn test_choose_spell_to_cast() {
        let player_id = EntityId::new(1);
        let mut controller = ZeroControllerV2::new(player_id);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let spells = vec![EntityId::new(20), EntityId::new(21), EntityId::new(22)];
        let chosen = controller.choose_spell_to_cast(&view, &spells);

        // Should choose the first spell
        assert!(chosen.is_some());
        let (spell_id, targets) = chosen.unwrap();
        assert_eq!(spell_id, EntityId::new(20));
        assert!(targets.is_empty());
    }

    #[test]
    fn test_choose_attackers() {
        let player_id = EntityId::new(1);
        let mut controller = ZeroControllerV2::new(player_id);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let creatures = vec![EntityId::new(30), EntityId::new(31), EntityId::new(32)];
        let attackers = controller.choose_attackers(&view, &creatures);

        // Should attack with all creatures
        assert_eq!(attackers.len(), 3);
        assert_eq!(attackers[0], EntityId::new(30));
        assert_eq!(attackers[1], EntityId::new(31));
        assert_eq!(attackers[2], EntityId::new(32));
    }

    #[test]
    fn test_choose_blockers() {
        let player_id = EntityId::new(1);
        let mut controller = ZeroControllerV2::new(player_id);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let blockers = vec![EntityId::new(40), EntityId::new(41)];
        let attackers = vec![EntityId::new(50), EntityId::new(51), EntityId::new(52)];
        let blocks = controller.choose_blockers(&view, &blockers, &attackers);

        // Should block first 2 attackers (limited by blocker count)
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0], (EntityId::new(40), EntityId::new(50)));
        assert_eq!(blocks[1], (EntityId::new(41), EntityId::new(51)));
    }

    #[test]
    fn test_choose_cards_to_discard() {
        let player_id = EntityId::new(1);
        let mut controller = ZeroControllerV2::new(player_id);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let hand = vec![
            EntityId::new(60),
            EntityId::new(61),
            EntityId::new(62),
            EntityId::new(63),
        ];

        let discards = controller.choose_cards_to_discard(&view, &hand, 2);

        // Should discard first 2 cards
        assert_eq!(discards.len(), 2);
        assert_eq!(discards[0], EntityId::new(60));
        assert_eq!(discards[1], EntityId::new(61));
    }

    #[test]
    fn test_tap_for_mana() {
        let player_id = EntityId::new(1);
        let mut controller = ZeroControllerV2::new(player_id);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let lands = vec![EntityId::new(70), EntityId::new(71)];
        let choice = controller.choose_card_to_tap_for_mana(&view, &lands);

        // Zero controller doesn't proactively tap for mana
        assert_eq!(choice, None);
    }

    #[test]
    fn test_wants_to_pass_priority() {
        let player_id = EntityId::new(1);
        let mut controller = ZeroControllerV2::new(player_id);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        // Zero controller always passes priority
        assert!(controller.wants_to_pass_priority(&view));
    }
}
