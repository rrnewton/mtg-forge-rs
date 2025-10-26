//! Fully safe bump-allocating logger
//!
//! This implementation is 100% safe Rust with no unsafe keyword usage.
//! It uses owned Strings in LogEntry and returns a guard type for iteration.

use crate::game::VerbosityLevel;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefCell};
use std::fmt::Write as FmtWrite;
use std::ops::Deref;

/// Output format for log messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    /// Human-readable text output (default)
    #[default]
    Text,
    /// Machine-readable JSON output (one object per line)
    Json,
}

/// A log entry with owned strings (no lifetime parameters)
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Verbosity level of this log entry
    pub level: VerbosityLevel,
    /// Log message (owned)
    pub message: String,
    /// Optional category (e.g., "controller_choice", "game_event")
    pub category: Option<String>,
}

/// Guard type that provides read-only access to log entries
///
/// This provides slice-like access to captured log entries.
pub struct LogGuard<'a> {
    guard: Ref<'a, Vec<LogEntry>>,
}

impl<'a> LogGuard<'a> {
    /// Get an iterator over log entries
    pub fn iter(&self) -> std::slice::Iter<'_, LogEntry> {
        self.guard.iter()
    }

    /// Get the number of log entries
    pub fn len(&self) -> usize {
        self.guard.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.guard.is_empty()
    }
}

// Deref to slice for convenient access
impl<'a> Deref for LogGuard<'a> {
    type Target = [LogEntry];

    fn deref(&self) -> &Self::Target {
        &self.guard // Auto-deref handles Ref -> Vec -> slice
    }
}

/// Centralized logger using bump allocation for temporary formatting
///
/// This logger avoids allocations during formatting by using a bump allocator
/// for temporary strings. LogEntries use owned Strings to avoid lifetime issues.
/// The implementation is 100% safe Rust with no unsafe code.
pub struct GameLogger {
    verbosity: VerbosityLevel,
    step_header_printed: bool,
    numeric_choices: bool,
    output_format: OutputFormat,
    capture_logs: bool,

    /// Bump allocator for temporary string formatting
    /// Reset after each format operation to avoid growth
    format_bump: RefCell<Bump>,

    /// Captured log entries (owned strings)
    log_buffer: RefCell<Vec<LogEntry>>,
}

impl GameLogger {
    /// Create a new logger with default verbosity (Normal)
    pub fn new() -> Self {
        GameLogger {
            verbosity: VerbosityLevel::default(),
            step_header_printed: false,
            numeric_choices: false,
            output_format: OutputFormat::default(),
            format_bump: RefCell::new(Bump::new()),
            log_buffer: RefCell::new(Vec::new()),
            capture_logs: false,
        }
    }

    /// Create a logger with specified verbosity
    pub fn with_verbosity(verbosity: VerbosityLevel) -> Self {
        GameLogger {
            verbosity,
            step_header_printed: false,
            numeric_choices: false,
            output_format: OutputFormat::default(),
            format_bump: RefCell::new(Bump::new()),
            log_buffer: RefCell::new(Vec::new()),
            capture_logs: false,
        }
    }

    /// Enable log capture to in-memory buffer
    pub fn enable_capture(&mut self) {
        self.capture_logs = true;
    }

    /// Disable log capture
    pub fn disable_capture(&mut self) {
        self.capture_logs = false;
    }

    /// Check if log capture is enabled
    pub fn is_capturing(&self) -> bool {
        self.capture_logs
    }

    /// Get access to captured log entries
    ///
    /// Returns a guard that derefs to `[LogEntry]`. You can iterate over it:
    ///
    /// # Example
    /// ```ignore
    /// let logs = logger.logs();
    /// for log in logs.iter() {
    ///     if log.message.contains("attack") {
    ///         println!("{}", log.message);
    ///     }
    /// }
    ///
    /// // Or count matching logs:
    /// let count = logger.logs().iter()
    ///     .filter(|log| log.message.contains("attack"))
    ///     .count();
    /// ```
    pub fn logs(&self) -> LogGuard<'_> {
        LogGuard {
            guard: self.log_buffer.borrow(),
        }
    }

    /// Clear the log buffer
    pub fn clear_logs(&mut self) {
        self.log_buffer.borrow_mut().clear();
        self.format_bump.borrow_mut().reset();
    }

    /// Set output format (Text or JSON)
    pub fn set_output_format(&mut self, format: OutputFormat) {
        self.output_format = format;
    }

    /// Get current output format
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
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

    /// Reset the step header flag
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

    /// Fast path for stdout logging
    #[inline]
    fn log_to_stdout(&self, level: VerbosityLevel, message: &str) {
        if level == VerbosityLevel::Minimal {
            println!("{}", message);
        } else {
            println!("  {}", message);
        }
    }

    /// Log at Silent level
    #[inline]
    pub fn silent(&self, _message: &str) {
        // Silent messages are never printed or captured
    }

    /// Log at Minimal level
    #[inline]
    pub fn minimal(&self, message: &str) {
        if VerbosityLevel::Minimal > self.verbosity && !self.capture_logs {
            return;
        }

        // Capture if enabled
        if self.capture_logs {
            self.log_buffer.borrow_mut().push(LogEntry {
                level: VerbosityLevel::Minimal,
                message: message.to_string(),
                category: None,
            });
        }

        // Output if verbosity allows
        if VerbosityLevel::Minimal <= self.verbosity {
            self.log_to_stdout(VerbosityLevel::Minimal, message);
        }
    }

    /// Log at Normal level
    #[inline]
    pub fn normal(&self, message: &str) {
        if VerbosityLevel::Normal > self.verbosity && !self.capture_logs {
            return;
        }

        // Capture if enabled
        if self.capture_logs {
            self.log_buffer.borrow_mut().push(LogEntry {
                level: VerbosityLevel::Normal,
                message: message.to_string(),
                category: None,
            });
        }

        // Output if verbosity allows
        if VerbosityLevel::Normal <= self.verbosity {
            self.log_to_stdout(VerbosityLevel::Normal, message);
        }
    }

    /// Log at Verbose level
    #[inline]
    pub fn verbose(&self, message: &str) {
        if VerbosityLevel::Verbose > self.verbosity && !self.capture_logs {
            return;
        }

        // Capture if enabled
        if self.capture_logs {
            self.log_buffer.borrow_mut().push(LogEntry {
                level: VerbosityLevel::Verbose,
                message: message.to_string(),
                category: None,
            });
        }

        // Output if verbosity allows
        if VerbosityLevel::Verbose <= self.verbosity {
            self.log_to_stdout(VerbosityLevel::Verbose, message);
        }
    }

    /// Log a controller decision at Normal level
    ///
    /// Uses bump allocator for temporary formatting to avoid intermediate allocations.
    #[inline]
    pub fn controller_choice(&self, controller_name: &str, message: &str) {
        let should_log = self.numeric_choices || self.verbosity >= VerbosityLevel::Normal;

        if !should_log && !self.capture_logs {
            return;
        }

        // Use bump allocator for temporary formatting
        let formatted = {
            let bump = self.format_bump.borrow();
            let mut temp = bumpalo::collections::String::new_in(&bump);
            write!(&mut temp, ">>> {}: {}", controller_name, message).unwrap();
            temp.to_string() // Convert to owned String
        };

        // Reset bump to avoid growth
        self.format_bump.borrow_mut().reset();

        // Capture if enabled
        if self.capture_logs {
            self.log_buffer.borrow_mut().push(LogEntry {
                level: VerbosityLevel::Normal,
                message: formatted.clone(),
                category: Some("controller_choice".to_string()),
            });
        }

        // Output if should_log
        if should_log {
            println!("  {}", formatted);
        }
    }
}

impl Default for GameLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for GameLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameLogger")
            .field("verbosity", &self.verbosity)
            .field("capture_logs", &self.capture_logs)
            .field("log_count", &self.log_buffer.borrow().len())
            .finish()
    }
}

impl Clone for GameLogger {
    fn clone(&self) -> Self {
        GameLogger {
            verbosity: self.verbosity,
            step_header_printed: self.step_header_printed,
            numeric_choices: self.numeric_choices,
            output_format: self.output_format,
            format_bump: RefCell::new(Bump::new()),
            log_buffer: RefCell::new(Vec::new()),
            capture_logs: self.capture_logs,
        }
    }
}

impl Serialize for GameLogger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("GameLogger", 4)?;
        state.serialize_field("verbosity", &self.verbosity)?;
        state.serialize_field("numeric_choices", &self.numeric_choices)?;
        state.serialize_field("output_format", &self.output_format)?;
        state.serialize_field("capture_logs", &self.capture_logs)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for GameLogger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct GameLoggerData {
            verbosity: VerbosityLevel,
            numeric_choices: bool,
            output_format: OutputFormat,
            capture_logs: bool,
        }

        let data = GameLoggerData::deserialize(deserializer)?;
        Ok(GameLogger {
            verbosity: data.verbosity,
            step_header_printed: false,
            numeric_choices: data.numeric_choices,
            output_format: data.output_format,
            format_bump: RefCell::new(Bump::new()),
            log_buffer: RefCell::new(Vec::new()),
            capture_logs: data.capture_logs,
        })
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
    fn test_log_capture() {
        let mut logger = GameLogger::new();
        logger.enable_capture();

        logger.normal("test message");
        logger.minimal("minimal message");

        let logs = logger.logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].message, "test message");
        assert_eq!(logs[1].message, "minimal message");
    }

    #[test]
    fn test_zero_copy_iteration() {
        let mut logger = GameLogger::new();
        logger.enable_capture();

        for i in 0..100 {
            logger.normal(&format!("message {}", i));
        }

        // Iterate without copying
        let count = logger
            .logs()
            .iter()
            .filter(|log| log.message.contains("5"))
            .count();

        // Should match: 5, 15, 25, ..., 95, 50-59
        assert!(count > 10);
    }
}
