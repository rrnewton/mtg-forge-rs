//! Core game state and turn structure

pub mod actions;
pub mod combat;
pub mod controller;
pub mod fixed_script_controller;
pub mod game_loop;
pub mod game_state_evaluator;
pub mod heuristic_controller;
pub mod interactive_controller;
pub mod logger;
pub mod mana_engine;
pub mod mana_payment;
pub mod phase;
pub mod random_controller;
pub mod replay_controller;
pub mod snapshot;
pub mod state;
pub mod state_hash;
pub mod stop_condition;
pub mod zero_controller;

#[cfg(test)]
mod controller_tests;
#[cfg(test)]
mod counter_tests;

pub use actions::GameAction;
pub use combat::CombatState;
pub use controller::{format_choice_menu, GameStateView, PlayerController};
pub use fixed_script_controller::FixedScriptController;
pub use game_loop::{GameEndReason, GameLoop, GameResult, VerbosityLevel};
pub use game_state_evaluator::{GameStateEvaluator, Score};
pub use heuristic_controller::HeuristicController;
pub use interactive_controller::InteractiveController;
pub use logger::{GameLogger, LogEntry, OutputFormat, OutputMode};
pub use mana_engine::{ManaCapacity, ManaEngine};
pub use mana_payment::{
    GreedyManaResolver, ManaColor, ManaPaymentResolver, ManaProduction, ManaSource,
    SimpleManaResolver,
};
pub use phase::{Phase, Step, TurnStructure};
pub use random_controller::RandomController;
pub use replay_controller::{ReplayChoice, ReplayController};
pub use snapshot::{ControllerState, GameSnapshot, SnapshotError};
pub use state::GameState;
pub use state_hash::{compute_state_hash, format_hash};
pub use stop_condition::{StopCondition, StopPlayer};
pub use zero_controller::ZeroController;
