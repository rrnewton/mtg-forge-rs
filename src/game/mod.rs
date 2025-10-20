//! Core game state and turn structure

pub mod actions;
pub mod controller;
pub mod game_loop;
pub mod phase;
pub mod random_controller;
pub mod scripted_controller;
pub mod state;

pub use actions::GameAction;
pub use controller::{GameStateView, PlayerAction, PlayerController};
pub use game_loop::{GameEndReason, GameLoop, GameResult};
pub use phase::{Phase, Step, TurnStructure};
pub use random_controller::RandomController;
pub use scripted_controller::ScriptedController;
pub use state::GameState;
