//! Core game state and turn structure

pub mod actions;
pub mod combat;
pub mod controller;
pub mod game_loop;
pub mod heuristic_controller;
pub mod interactive_controller;
pub mod logger;
pub mod mana_engine;
pub mod phase;
pub mod random_controller;
pub mod state;
pub mod zero_controller;

pub use actions::GameAction;
pub use combat::CombatState;
pub use controller::{GameStateView, PlayerController};
pub use game_loop::{GameEndReason, GameLoop, GameResult, VerbosityLevel};
pub use heuristic_controller::HeuristicController;
pub use interactive_controller::InteractiveController;
pub use logger::{GameLogger, LogEntry, OutputFormat};
pub use mana_engine::{ManaCapacity, ManaEngine};
pub use phase::{Phase, Step, TurnStructure};
pub use random_controller::RandomController;
pub use state::GameState;
pub use zero_controller::ZeroController;
