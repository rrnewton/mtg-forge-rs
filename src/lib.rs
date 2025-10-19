//! MTG Forge - High-performance Rust port for AI research
//!
//! This is a port of the MTG Forge game engine from Java to Rust,
//! optimized for efficient tree search and AI gameplay.

pub mod core;
pub mod game;
pub mod zones;
pub mod loader;
pub mod undo;
pub mod error;

pub use error::{MtgError, Result};
