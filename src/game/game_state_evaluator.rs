//! Game state evaluation for heuristic AI
//!
//! This module provides holistic board evaluation, scoring the overall game state
//! from a player's perspective. This is a faithful port of Java Forge's
//! GameStateEvaluator.java which is used by the simulation-based AI.
//!
//! Reference: forge-java/forge-ai/src/main/java/forge/ai/simulation/GameStateEvaluator.java

use crate::core::{Card, PlayerId};
use crate::game::controller::GameStateView;
use crate::game::heuristic_controller::HeuristicController;

/// Score representing the value of a game state
///
/// Reference: GameStateEvaluator.Score (lines 297-320)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score {
    /// Overall board value from AI's perspective
    /// Positive = good for AI, Negative = good for opponent
    /// MAX_VALUE = AI wins, MIN_VALUE = AI loses
    pub value: i32,

    /// Score excluding summon sick creatures
    /// Used to encourage holding creatures until Main Phase 2
    /// if they don't provide immediate value
    pub summon_sick_value: i32,
}

impl Score {
    /// Create a new score with the same value for both metrics
    pub fn new(value: i32) -> Self {
        Score {
            value,
            summon_sick_value: value,
        }
    }

    /// Create a score with different summon sick value
    pub fn with_summon_sick(value: i32, summon_sick_value: i32) -> Self {
        Score {
            value,
            summon_sick_value,
        }
    }

    /// Score indicating the AI has won
    pub const WIN: Score = Score {
        value: i32::MAX,
        summon_sick_value: i32::MAX,
    };

    /// Score indicating the AI has lost
    pub const LOSS: Score = Score {
        value: i32::MIN,
        summon_sick_value: i32::MIN,
    };
}

/// Game state evaluator for heuristic AI
///
/// Provides holistic board evaluation by scoring:
/// - Life totals
/// - Cards in hand
/// - Battlefield permanents (creatures, lands, etc.)
/// - Mana base quality
///
/// Reference: GameStateEvaluator.java
pub struct GameStateEvaluator {
    /// Creature evaluator (reuse from HeuristicController)
    creature_eval: HeuristicController,
}

impl GameStateEvaluator {
    /// Create a new game state evaluator
    pub fn new(player_id: PlayerId) -> Self {
        GameStateEvaluator {
            creature_eval: HeuristicController::new(player_id),
        }
    }

    /// Evaluate the current game state from the AI's perspective
    ///
    /// Returns a Score where:
    /// - Positive values favor the AI
    /// - Negative values favor opponents
    /// - Score::WIN if AI has won
    /// - Score::LOSS if AI has lost
    ///
    /// Reference: GameStateEvaluator.getScoreForGameState() (lines 86-100)
    pub fn evaluate_game_state(&self, view: &GameStateView, ai_player: PlayerId) -> Score {
        // TODO: Check if game is over and return WIN/LOSS

        // TODO: Simulate upcoming combat to see if it's lethal
        // For now, just evaluate the current board state

        self.evaluate_game_state_impl(view, ai_player)
    }

    /// Internal implementation of game state evaluation
    ///
    /// Reference: GameStateEvaluator.getScoreForGameStateImpl() (lines 102-174)
    fn evaluate_game_state_impl(&self, view: &GameStateView, ai_player: PlayerId) -> Score {
        let mut score = 0;

        // Count cards in hand
        // Java: +5 per AI card, -4 per opponent card (lines 108-123)
        let my_hand_size = view.player_hand(ai_player).len() as i32;
        let opponent_hand_size = self.get_opponent_hand_size(view, ai_player);

        score += 5 * my_hand_size - 4 * opponent_hand_size;

        // Life totals
        // Java: +2 per AI life, -2 per opponent life (lines 124-133)
        let my_life = view.life();
        let opponent_life = self.get_opponent_life(view, ai_player);

        score += 2 * my_life;
        score -= 2 * opponent_life;

        // Evaluate mana base quality
        // TODO(vc-2): Port evalManaBase() (lines 176-216)
        // For now, skip mana base evaluation

        // Evaluate battlefield permanents
        // Java: Loop through all battlefield cards, evaluate each (lines 148-170)
        let mut summon_sick_score = score;

        for &card_id in view.battlefield() {
            if let Some(card) = view.get_card(card_id) {
                let value = self.evaluate_card(card);

                // TODO(vc-3): Track summon sickness properly
                // For now, treat all creatures the same
                let summon_sick_value = value;

                // Cards owned by AI add to score, opponent cards subtract
                if card.owner == ai_player {
                    score += value;
                    summon_sick_score += summon_sick_value;
                } else {
                    score -= value;
                    summon_sick_score -= summon_sick_value;
                }
            }
        }

        Score::with_summon_sick(score, summon_sick_score)
    }

    /// Evaluate a single card on the battlefield
    ///
    /// Reference: GameStateEvaluator.evalCard() (lines 218-238)
    fn evaluate_card(&self, card: &Card) -> i32 {
        if card.is_creature() {
            self.creature_eval.evaluate_creature(card)
        } else if card.is_land() {
            Self::evaluate_land(card)
        } else if card.is_enchantment() {
            // TODO(vc-4): Properly evaluate enchantments
            // Java: Should only provide value based on what they enchant (lines 224-228)
            0
        } else {
            // Other permanents (artifacts, planeswalkers, etc.)
            // Java: 50 + 30 * CMC (lines 232-236)
            let cmc = card.mana_cost.cmc() as i32;
            50 + 30 * cmc
        }
    }

    /// Evaluate a land card
    ///
    /// Reference: GameStateEvaluator.evaluateLand() (lines 240-285)
    pub fn evaluate_land(card: &Card) -> i32 {
        let mut value = 3;

        // TODO(vc-5): Detailed land evaluation
        // - +100 per mana produced
        // - +3 per color produced
        // - +25-50 for utility abilities (manlands, etc.)
        // - +6 per static ability
        //
        // For now, use a simple heuristic:
        // Basic lands are worth ~100 (base 3 + mana production)
        // Non-basics might be worth more for fixing

        // Check for activated abilities (very rough approximation)
        if !card.activated_abilities.is_empty() {
            // Has abilities, likely produces mana
            value += 100;

            // If it has multiple abilities, it might be a utility land
            if card.activated_abilities.len() > 1 {
                value += 25;
            }
        }

        value
    }

    /// Get total hand size for all opponents
    fn get_opponent_hand_size(&self, _view: &GameStateView, ai_player: PlayerId) -> i32 {
        // TODO: Support multiplayer (iterate all opponents)
        // For now, assume 2-player game
        let opponent_id = if ai_player.as_u32() == 0 {
            PlayerId::new(1)
        } else {
            PlayerId::new(0)
        };
        _view.player_hand(opponent_id).len() as i32
    }

    /// Get total life for all opponents
    fn get_opponent_life(&self, _view: &GameStateView, _ai_player: PlayerId) -> i32 {
        // TODO: Support multiplayer (iterate all opponents)
        // For now, assume 2-player game and hardcode access
        // We need a way to get opponent life from view
        // For now, return a placeholder
        20 // TODO(vc-6): Need GameStateView.opponent_life()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_creation() {
        let score = Score::new(100);
        assert_eq!(score.value, 100);
        assert_eq!(score.summon_sick_value, 100);

        let score2 = Score::with_summon_sick(150, 100);
        assert_eq!(score2.value, 150);
        assert_eq!(score2.summon_sick_value, 100);
    }

    #[test]
    fn test_win_loss_scores() {
        assert_eq!(Score::WIN.value, i32::MAX);
        assert_eq!(Score::LOSS.value, i32::MIN);
    }
}
