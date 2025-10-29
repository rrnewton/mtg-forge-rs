//! Replay controller that replays logged choices then delegates to another controller
//!
//! This controller is used for snapshot resume: it replays a sequence of predetermined
//! choices (from the snapshot's intra-turn choice log), then hands control to the
//! wrapped controller for subsequent choices.

use crate::core::{CardId, ManaCost, PlayerId, SpellAbility};
use crate::game::controller::{GameStateView, PlayerController};
use smallvec::SmallVec;

/// A single recorded choice from a controller
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ReplayChoice {
    /// Choice of spell ability to play (or None to pass priority)
    SpellAbility(Option<SpellAbility>),
    /// Choice of targets for a spell
    Targets(SmallVec<[CardId; 4]>),
    /// Choice of mana sources to tap
    ManaSources(SmallVec<[CardId; 8]>),
    /// Choice of attackers
    Attackers(SmallVec<[CardId; 8]>),
    /// Choice of blockers
    Blockers(SmallVec<[(CardId, CardId); 8]>),
    /// Choice of damage assignment order
    DamageOrder(SmallVec<[CardId; 4]>),
    /// Choice of cards to discard
    Discard(SmallVec<[CardId; 7]>),
}

/// Controller that replays a sequence of choices then delegates to another controller
///
/// This is used for snapshot resume. The replay controller:
/// 1. Plays back a predetermined sequence of choices from the snapshot
/// 2. Once all replay choices are exhausted, delegates to the wrapped controller
///
/// ## Usage
///
/// ```rust,ignore
/// // Create a controller with replay choices
/// let replay_choices = vec![
///     ReplayChoice::SpellAbility(Some(SpellAbility::PlayLand { card_id: CardId::new(1) })),
///     ReplayChoice::Targets(SmallVec::new()),
/// ];
///
/// let base_controller = RandomController::with_seed(player_id, 42);
/// let mut replay_controller = ReplayController::new(player_id, base_controller, replay_choices);
///
/// // Use replay_controller normally - it will replay choices then delegate
/// ```
pub struct ReplayController {
    player_id: PlayerId,
    /// The wrapped controller to delegate to after replay is exhausted
    inner: Box<dyn PlayerController>,
    /// Queue of choices to replay (consumed from front)
    replay_choices: Vec<ReplayChoice>,
    /// Current index in the replay queue
    replay_index: usize,
}

impl ReplayController {
    /// Create a new replay controller
    ///
    /// # Arguments
    /// * `player_id` - The player ID this controller manages
    /// * `inner` - The controller to delegate to after replay is exhausted
    /// * `replay_choices` - Sequence of choices to replay before delegating
    pub fn new(
        player_id: PlayerId,
        inner: Box<dyn PlayerController>,
        replay_choices: Vec<ReplayChoice>,
    ) -> Self {
        ReplayController {
            player_id,
            inner,
            replay_choices,
            replay_index: 0,
        }
    }

    /// Check if we have more replay choices to consume
    fn has_replay_choice(&self) -> bool {
        self.replay_index < self.replay_choices.len()
    }

    /// Consume the next replay choice of the expected type
    ///
    /// Returns the choice if available and of the correct type, otherwise None.
    fn consume_replay_choice<F, T>(&mut self, extract: F) -> Option<T>
    where
        F: FnOnce(&ReplayChoice) -> Option<T>,
    {
        if !self.has_replay_choice() {
            return None;
        }

        let choice = &self.replay_choices[self.replay_index];
        if let Some(value) = extract(choice) {
            self.replay_index += 1;
            Some(value)
        } else {
            // Type mismatch - this shouldn't happen if replay log is correct
            eprintln!(
                "WARNING: Replay choice type mismatch at index {}. Expected different type, got {:?}",
                self.replay_index, choice
            );
            None
        }
    }
}

impl PlayerController for ReplayController {
    fn player_id(&self) -> PlayerId {
        self.player_id
    }

    fn choose_spell_ability_to_play(
        &mut self,
        view: &GameStateView,
        available: &[SpellAbility],
    ) -> Option<SpellAbility> {
        // Try to consume a replay choice first
        if let Some(choice) = self.consume_replay_choice(|c| {
            if let ReplayChoice::SpellAbility(opt) = c {
                Some(opt.clone())
            } else {
                None
            }
        }) {
            return choice;
        }

        // No replay choice available, delegate to inner controller
        self.inner.choose_spell_ability_to_play(view, available)
    }

    fn choose_targets(
        &mut self,
        view: &GameStateView,
        spell: CardId,
        valid_targets: &[CardId],
    ) -> SmallVec<[CardId; 4]> {
        // Try to consume a replay choice first
        if let Some(targets) = self.consume_replay_choice(|c| {
            if let ReplayChoice::Targets(t) = c {
                Some(t.clone())
            } else {
                None
            }
        }) {
            return targets;
        }

        // No replay choice available, delegate to inner controller
        self.inner.choose_targets(view, spell, valid_targets)
    }

    fn choose_mana_sources_to_pay(
        &mut self,
        view: &GameStateView,
        cost: &ManaCost,
        available_sources: &[CardId],
    ) -> SmallVec<[CardId; 8]> {
        // Try to consume a replay choice first
        if let Some(sources) = self.consume_replay_choice(|c| {
            if let ReplayChoice::ManaSources(s) = c {
                Some(s.clone())
            } else {
                None
            }
        }) {
            return sources;
        }

        // No replay choice available, delegate to inner controller
        self.inner
            .choose_mana_sources_to_pay(view, cost, available_sources)
    }

    fn choose_attackers(
        &mut self,
        view: &GameStateView,
        available_creatures: &[CardId],
    ) -> SmallVec<[CardId; 8]> {
        // Try to consume a replay choice first
        if let Some(attackers) = self.consume_replay_choice(|c| {
            if let ReplayChoice::Attackers(a) = c {
                Some(a.clone())
            } else {
                None
            }
        }) {
            return attackers;
        }

        // No replay choice available, delegate to inner controller
        self.inner.choose_attackers(view, available_creatures)
    }

    fn choose_blockers(
        &mut self,
        view: &GameStateView,
        available_blockers: &[CardId],
        attackers: &[CardId],
    ) -> SmallVec<[(CardId, CardId); 8]> {
        // Try to consume a replay choice first
        if let Some(blockers) = self.consume_replay_choice(|c| {
            if let ReplayChoice::Blockers(b) = c {
                Some(b.clone())
            } else {
                None
            }
        }) {
            return blockers;
        }

        // No replay choice available, delegate to inner controller
        self.inner
            .choose_blockers(view, available_blockers, attackers)
    }

    fn choose_damage_assignment_order(
        &mut self,
        view: &GameStateView,
        attacker: CardId,
        blockers: &[CardId],
    ) -> SmallVec<[CardId; 4]> {
        // Try to consume a replay choice first
        if let Some(order) = self.consume_replay_choice(|c| {
            if let ReplayChoice::DamageOrder(o) = c {
                Some(o.clone())
            } else {
                None
            }
        }) {
            return order;
        }

        // No replay choice available, delegate to inner controller
        self.inner
            .choose_damage_assignment_order(view, attacker, blockers)
    }

    fn choose_cards_to_discard(
        &mut self,
        view: &GameStateView,
        hand: &[CardId],
        count: usize,
    ) -> SmallVec<[CardId; 7]> {
        // Try to consume a replay choice first
        if let Some(discard) = self.consume_replay_choice(|c| {
            if let ReplayChoice::Discard(d) = c {
                Some(d.clone())
            } else {
                None
            }
        }) {
            return discard;
        }

        // No replay choice available, delegate to inner controller
        self.inner.choose_cards_to_discard(view, hand, count)
    }

    fn on_priority_passed(&mut self, view: &GameStateView) {
        // Always delegate notifications to inner controller
        self.inner.on_priority_passed(view);
    }

    fn on_game_end(&mut self, view: &GameStateView, won: bool) {
        // Always delegate notifications to inner controller
        self.inner.on_game_end(view, won);
    }

    fn get_snapshot_state(&self) -> Option<serde_json::Value> {
        // Delegate to inner controller for state serialization
        // This allows the wrapped controller (RandomController, FixedScriptController, etc.)
        // to properly save its state even when wrapped in a ReplayController
        self.inner.get_snapshot_state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ZeroController;

    #[test]
    fn test_replay_controller_exhausts_choices() {
        let player_id = PlayerId::new(1);
        let inner = Box::new(ZeroController::new(player_id));

        let replay_choices = vec![
            ReplayChoice::SpellAbility(Some(SpellAbility::PlayLand {
                card_id: CardId::new(10),
            })),
            ReplayChoice::SpellAbility(None), // Pass priority
        ];

        let mut replay = ReplayController::new(player_id, inner, replay_choices);

        // Create a minimal game state for testing
        let game = crate::game::GameState::new_two_player(
            "Player 1".to_string(),
            "Player 2".to_string(),
            20,
        );
        let view = crate::game::GameStateView::new(&game, player_id);

        // First call should return the replayed choice
        assert!(replay.has_replay_choice());
        let choice1 = replay.choose_spell_ability_to_play(&view, &[]);
        assert!(choice1.is_some());

        // Second call should return the second replayed choice
        assert!(replay.has_replay_choice());
        let choice2 = replay.choose_spell_ability_to_play(&view, &[]);
        assert!(choice2.is_none()); // Second choice was None (pass priority)

        // After exhausting replay choices, should delegate to inner controller
        assert!(!replay.has_replay_choice());
    }

    #[test]
    fn test_replay_controller_delegates_after_exhaustion() {
        let player_id = PlayerId::new(1);
        let inner = Box::new(ZeroController::new(player_id));

        // Empty replay choices - should immediately delegate
        let replay_choices = vec![];

        let replay = ReplayController::new(player_id, inner, replay_choices);
        assert!(!replay.has_replay_choice());
    }
}
