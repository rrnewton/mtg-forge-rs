//! Core game state and turn structure

pub mod phase;
pub mod state;

pub use phase::{Phase, Step, TurnStructure};
pub use state::GameState;
