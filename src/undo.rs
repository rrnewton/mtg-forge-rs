//! Undo log for efficient game tree search
//!
//! This module provides a transaction log of game actions that can be
//! rewound to efficiently explore the game tree without expensive deep copies.

use crate::core::{CardId, CounterType, PlayerId};
use crate::zones::Zone;
use serde::{Deserialize, Serialize};

/// Atomic game actions that can be logged and undone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAction {
    /// Move a card between zones
    MoveCard {
        card_id: CardId,
        from_zone: Zone,
        to_zone: Zone,
        owner: PlayerId,
    },

    /// Tap/untap a permanent
    TapCard { card_id: CardId, tapped: bool },

    /// Modify life total
    ModifyLife { player_id: PlayerId, amount: i32 },

    /// Add mana to pool
    AddMana {
        player_id: PlayerId,
        color: crate::core::Color,
    },

    /// Empty mana pool
    EmptyManaPool { player_id: PlayerId },

    /// Add counter to card
    AddCounter {
        card_id: CardId,
        counter_type: CounterType,
        amount: u8,
    },

    /// Remove counter from card
    RemoveCounter {
        card_id: CardId,
        counter_type: CounterType,
        amount: u8,
    },

    /// Advance game step
    AdvanceStep {
        from_step: crate::game::Step,
        to_step: crate::game::Step,
    },

    /// Change turn
    ChangeTurn {
        from_player: PlayerId,
        to_player: PlayerId,
        turn_number: u32,
    },

    /// Mark a choice point (for tree search)
    ChoicePoint { player_id: PlayerId, choice_id: u32 },
}

/// Undo log for tracking and rewinding game actions
///
/// This allows efficient tree search by mutating game state forward
/// and then rewinding via the log, instead of expensive deep copies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoLog {
    /// Stack of actions (most recent at end)
    actions: Vec<GameAction>,

    /// Is logging enabled? (can be compiled out for replay benchmarks)
    enabled: bool,

    /// Mark positions for choice points
    choice_points: Vec<usize>,
}

impl UndoLog {
    pub fn new() -> Self {
        UndoLog {
            actions: Vec::new(),
            enabled: true,
            choice_points: Vec::new(),
        }
    }

    /// Create a disabled undo log (for benchmarking)
    pub fn disabled() -> Self {
        UndoLog {
            actions: Vec::new(),
            enabled: false,
            choice_points: Vec::new(),
        }
    }

    /// Log an action
    pub fn log(&mut self, action: GameAction) {
        if self.enabled {
            self.actions.push(action);
        }
    }

    /// Mark a choice point in the log
    pub fn mark_choice_point(&mut self) {
        if self.enabled {
            self.choice_points.push(self.actions.len());
        }
    }

    /// Get the most recent action without removing it
    pub fn peek(&self) -> Option<&GameAction> {
        self.actions.last()
    }

    /// Pop and return the most recent action
    pub fn pop(&mut self) -> Option<GameAction> {
        self.actions.pop()
    }

    /// Get number of actions in log
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Clear all actions up to the most recent choice point
    pub fn rewind_to_choice_point(&mut self) {
        if let Some(checkpoint) = self.choice_points.pop() {
            self.actions.truncate(checkpoint);
        }
    }

    /// Clear the entire log
    pub fn clear(&mut self) {
        self.actions.clear();
        self.choice_points.clear();
    }

    /// Get all actions (for debugging/serialization)
    pub fn actions(&self) -> &[GameAction] {
        &self.actions
    }
}

impl Default for UndoLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_log() {
        let mut log = UndoLog::new();
        assert_eq!(log.len(), 0);

        let action = GameAction::ModifyLife {
            player_id: PlayerId::new(1),
            amount: -3,
        };

        log.log(action.clone());
        assert_eq!(log.len(), 1);

        let popped = log.pop().unwrap();
        assert!(matches!(popped, GameAction::ModifyLife { .. }));
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_choice_points() {
        let mut log = UndoLog::new();

        log.log(GameAction::ModifyLife {
            player_id: PlayerId::new(1),
            amount: -1,
        });
        log.log(GameAction::ModifyLife {
            player_id: PlayerId::new(1),
            amount: -1,
        });

        log.mark_choice_point();

        log.log(GameAction::ModifyLife {
            player_id: PlayerId::new(1),
            amount: -1,
        });
        log.log(GameAction::ModifyLife {
            player_id: PlayerId::new(1),
            amount: -1,
        });

        assert_eq!(log.len(), 4);

        log.rewind_to_choice_point();
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_disabled_log() {
        let mut log = UndoLog::disabled();

        log.log(GameAction::ModifyLife {
            player_id: PlayerId::new(1),
            amount: -1,
        });

        assert_eq!(log.len(), 0); // Nothing logged when disabled
    }
}
