//! Centralized game logging
//!
//! This module provides a centralized logger for game events, controller decisions,
//! and other gameplay information. The logger internally tracks verbosity level
//! and can be accessed by different parts of the game (controllers, game loop, etc.)

use crate::game::VerbosityLevel;
use serde::{Deserialize, Serialize};

/// Centralized logger for game events
///
/// This logger is stored in GameState and can be accessed via GameStateView.
/// It internally tracks the verbosity level and provides methods for logging
/// at different verbosity levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLogger {
    verbosity: VerbosityLevel,
    /// Track if we've printed the step header (for lazy printing)
    #[serde(skip)]
    step_header_printed: bool,
    /// Enable numeric-only choice format (for Java Forge comparison)
    numeric_choices: bool,
}

impl GameLogger {
    /// Create a new logger with default verbosity (Normal)
    pub fn new() -> Self {
        GameLogger {
            verbosity: VerbosityLevel::default(),
            step_header_printed: false,
            numeric_choices: false,
        }
    }

    /// Create a logger with specified verbosity
    pub fn with_verbosity(verbosity: VerbosityLevel) -> Self {
        GameLogger {
            verbosity,
            step_header_printed: false,
            numeric_choices: false,
        }
    }

    /// Enable numeric-only choice logging
    pub fn set_numeric_choices(&mut self, enabled: bool) {
        self.numeric_choices = enabled;
    }

    /// Check if numeric choices mode is enabled
    pub fn numeric_choices_enabled(&self) -> bool {
        self.numeric_choices
    }

    /// Get current verbosity level
    pub fn verbosity(&self) -> VerbosityLevel {
        self.verbosity
    }

    /// Set verbosity level
    pub fn set_verbosity(&mut self, verbosity: VerbosityLevel) {
        self.verbosity = verbosity;
    }

    /// Reset the step header flag (called when starting a new step)
    pub fn reset_step_header(&mut self) {
        self.step_header_printed = false;
    }

    /// Mark that step header has been printed
    pub fn mark_step_header_printed(&mut self) {
        self.step_header_printed = true;
    }

    /// Check if step header has been printed
    pub fn step_header_printed(&self) -> bool {
        self.step_header_printed
    }

    /// Log at Silent level (always suppressed - this is just for API consistency)
    #[inline]
    pub fn silent(&self, _message: &str) {
        // Silent messages are never printed
    }

    /// Log at Minimal level (game outcomes, major events)
    #[inline]
    pub fn minimal(&self, message: &str) {
        if self.verbosity >= VerbosityLevel::Minimal {
            println!("{message}");
        }
    }

    /// Log at Normal level (turns, steps, key actions)
    #[inline]
    pub fn normal(&self, message: &str) {
        if self.verbosity >= VerbosityLevel::Normal {
            println!("  {message}");
        }
    }

    /// Log at Verbose level (all actions and state changes)
    #[inline]
    pub fn verbose(&self, message: &str) {
        if self.verbosity >= VerbosityLevel::Verbose {
            println!("  {message}");
        }
    }

    /// Log a controller decision at Normal level
    ///
    /// This is a convenience method for logging AI/controller choices.
    /// If numeric_choices mode is enabled, choices are always logged regardless of verbosity.
    #[inline]
    pub fn controller_choice(&self, controller_name: &str, message: &str) {
        if self.numeric_choices || self.verbosity >= VerbosityLevel::Normal {
            println!("  >>> {controller_name}: {message}");
        }
    }
}

impl Default for GameLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = GameLogger::new();
        assert_eq!(logger.verbosity(), VerbosityLevel::Normal);
    }

    #[test]
    fn test_logger_with_verbosity() {
        let logger = GameLogger::with_verbosity(VerbosityLevel::Silent);
        assert_eq!(logger.verbosity(), VerbosityLevel::Silent);
    }

    #[test]
    fn test_set_verbosity() {
        let mut logger = GameLogger::new();
        logger.set_verbosity(VerbosityLevel::Verbose);
        assert_eq!(logger.verbosity(), VerbosityLevel::Verbose);
    }

    #[test]
    fn test_step_header_tracking() {
        let mut logger = GameLogger::new();
        assert!(!logger.step_header_printed());

        logger.mark_step_header_printed();
        assert!(logger.step_header_printed());

        logger.reset_step_header();
        assert!(!logger.step_header_printed());
    }
}
