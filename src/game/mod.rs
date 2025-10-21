//! Core game state and turn structure

pub mod actions;
pub mod combat;
pub mod controller;
pub mod game_loop;
pub mod phase;
pub mod random_controller;
pub mod scripted_controller;
pub mod state;
pub mod zero_controller;

pub use actions::GameAction;
pub use combat::CombatState;
pub use controller::{GameStateView, PlayerAction, PlayerController};
pub use game_loop::{GameEndReason, GameLoop, GameResult, VerbosityLevel};
pub use phase::{Phase, Step, TurnStructure};
pub use random_controller::RandomController;
pub use scripted_controller::ScriptedController;
pub use state::GameState;
pub use zero_controller::ZeroController;
