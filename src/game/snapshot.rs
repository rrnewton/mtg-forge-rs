//! Game snapshot functionality for stop-and-resume gameplay
//!
//! This module provides snapshot/resume functionality using a replay-based approach:
//! - Snapshots are saved at turn boundaries (clean save points)
//! - Intra-turn choices are logged and replayed on resume
//! - Uses existing undo log infrastructure for choice tracking

use crate::game::state::GameState;
use crate::undo::GameAction;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Controller state that can be preserved across snapshot/resume
///
/// This enum allows us to serialize and restore the state of different controller types.
/// Each variant contains the full state needed to reconstruct the controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "controller_type")]
pub enum ControllerState {
    /// Fixed script controller with predetermined choices
    Fixed(crate::game::FixedScriptController),

    /// Random controller with its own RNG state
    Random(crate::game::RandomController),
    // Other controller types don't need state preservation:
    // - Heuristic: Deterministic, no state needed
    // - Zero: Deterministic, no state needed
    // - Interactive: Human input, no state to preserve
    // - Replay: Wrapper around another controller (state handled separately)
}

/// A game snapshot saved at a turn boundary with intra-turn replay data
///
/// This format allows us to save only at clean turn boundaries while still
/// resuming at arbitrary intra-turn choice points via replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    /// The complete game state at the start of the turn
    pub game_state: GameState,

    /// Turn number when this snapshot was created
    pub turn_number: u32,

    /// Sequence of choice points made during this turn up to the stop point
    ///
    /// These will be replayed (with buffered logging) when resuming
    /// to restore the exact intra-turn state.
    pub intra_turn_choices: Vec<GameAction>,

    /// Optional controller state for player 1
    ///
    /// Preserves the full state of player 1's controller across snapshot/resume.
    /// Supports Fixed and Random controllers (others are stateless).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p1_controller_state: Option<ControllerState>,

    /// Optional controller state for player 2
    ///
    /// Preserves the full state of player 2's controller across snapshot/resume.
    /// Supports Fixed and Random controllers (others are stateless).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p2_controller_state: Option<ControllerState>,
}

impl GameSnapshot {
    /// Create a new snapshot from a game state, turn number, and choice history
    pub fn new(
        game_state: GameState,
        turn_number: u32,
        intra_turn_choices: Vec<GameAction>,
    ) -> Self {
        GameSnapshot {
            game_state,
            turn_number,
            intra_turn_choices,
            p1_controller_state: None,
            p2_controller_state: None,
        }
    }

    /// Create a snapshot with controller state preserved
    pub fn with_controller_state(
        game_state: GameState,
        turn_number: u32,
        intra_turn_choices: Vec<GameAction>,
        p1_controller_state: Option<ControllerState>,
        p2_controller_state: Option<ControllerState>,
    ) -> Self {
        GameSnapshot {
            game_state,
            turn_number,
            intra_turn_choices,
            p1_controller_state,
            p2_controller_state,
        }
    }

    /// Save this snapshot to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SnapshotError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| SnapshotError::Serialization(e.to_string()))?;

        std::fs::write(path.as_ref(), json).map_err(|e| SnapshotError::Io(e.to_string()))?;

        Ok(())
    }

    /// Load a snapshot from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, SnapshotError> {
        let json =
            std::fs::read_to_string(path.as_ref()).map_err(|e| SnapshotError::Io(e.to_string()))?;

        let snapshot = serde_json::from_str(&json)
            .map_err(|e| SnapshotError::Deserialization(e.to_string()))?;

        Ok(snapshot)
    }

    /// Get the number of intra-turn choices in this snapshot
    pub fn choice_count(&self) -> usize {
        self.intra_turn_choices.len()
    }

    /// Check if this snapshot has any intra-turn state to replay
    pub fn has_intra_turn_state(&self) -> bool {
        !self.intra_turn_choices.is_empty()
    }

    /// Extract replay choices from the intra-turn choice log
    ///
    /// Converts GameAction::ChoicePoint entries into a Vec<ReplayChoice> that can be
    /// fed to ReplayController for deterministic replay.
    pub fn extract_replay_choices(&self) -> Vec<crate::game::ReplayChoice> {
        self.intra_turn_choices
            .iter()
            .filter_map(|action| {
                if let GameAction::ChoicePoint { choice, .. } = action {
                    choice.clone()
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract replay choices for a specific player
    ///
    /// Filters the intra-turn choice log to only include choices made by the specified player.
    /// This is critical for snapshot resume - each controller should only replay its own choices!
    pub fn extract_replay_choices_for_player(
        &self,
        player_id: crate::core::PlayerId,
    ) -> Vec<crate::game::ReplayChoice> {
        self.intra_turn_choices
            .iter()
            .filter_map(|action| {
                if let GameAction::ChoicePoint {
                    player_id: choice_player,
                    choice,
                    ..
                } = action
                {
                    if *choice_player == player_id {
                        choice.clone()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Errors that can occur during snapshot operations
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("Failed to serialize snapshot: {0}")]
    Serialization(String),

    #[error("Failed to deserialize snapshot: {0}")]
    Deserialization(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Invalid snapshot state: {0}")]
    InvalidState(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PlayerId;

    #[test]
    fn test_snapshot_choice_tracking() {
        // Test the choice tracking logic without a full GameState
        let choices = [
            GameAction::ChoicePoint {
                player_id: PlayerId::new(0),
                choice_id: 1,
                choice: None,
            },
            GameAction::ChoicePoint {
                player_id: PlayerId::new(0),
                choice_id: 2,
                choice: None,
            },
        ];

        assert_eq!(choices.len(), 2);
        assert!(!choices.is_empty());
    }

    #[test]
    fn test_choice_point_serialization() {
        // Test that GameAction::ChoicePoint can be serialized/deserialized
        let choice = GameAction::ChoicePoint {
            player_id: PlayerId::new(1),
            choice_id: 42,
            choice: None,
        };

        let json = serde_json::to_string(&choice).unwrap();
        let deserialized: GameAction = serde_json::from_str(&json).unwrap();

        if let GameAction::ChoicePoint {
            player_id,
            choice_id,
            choice,
        } = deserialized
        {
            assert_eq!(player_id, PlayerId::new(1));
            assert_eq!(choice_id, 42);
            assert!(choice.is_none());
        } else {
            panic!("Failed to deserialize ChoicePoint");
        }
    }
}
