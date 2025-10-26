//! Bump-allocating in-memory logger
//!
//! This module provides a high-performance logger using bumpalo for arena allocation.
//! Log messages are stored in a bump allocator, avoiding individual heap allocations.

use crate::game::VerbosityLevel;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt::Write as FmtWrite;

/// Output format for log messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    /// Human-readable text output (default)
    #[default]
    Text,
    /// Machine-readable JSON output (one object per line)
    Json,
}

/// A log entry with string slices borrowing from the bump allocator
///
/// The lifetime 'a is tied to the logger's bump allocator lifetime.
#[derive(Debug)]
pub struct LogEntry<'a> {
    /// Verbosity level of this log entry
    pub level: VerbosityLevel,
    /// Log message (borrowed from bump allocator)
    pub message: &'a str,
    /// Optional category (e.g., "controller_choice", "game_event")
    pub category: Option<&'a str>,
}

/// Centralized logger using bump allocation for log storage
///
/// This logger uses a bump allocator to store log entries, avoiding per-entry
/// heap allocations. Clients access logs via iteration, not by copying.
pub struct GameLogger {
    verbosity: VerbosityLevel,
    step_header_printed: bool,
    numeric_choices: bool,
    output_format: OutputFormat,

    /// Bump allocator for log storage
    /// All log message strings are allocated here
    bump: RefCell<Bump>,

    /// Pointers to log entries in the bump allocator
    /// SAFETY: These pointers remain valid as long as the bump allocator lives
    /// and we never reset it until clear_logs() is called
    log_entries: RefCell<Vec<*const LogEntry<'static>>>,

    capture_logs: bool,
}

// SAFETY: The logger is Send if we can guarantee the bump allocator and pointers
// are not accessed concurrently. We use RefCell for interior mutability.
// The 'static lifetime in log_entries is a lie - they're actually 'bump lifetime,
// but we manage this carefully to ensure safety.
unsafe impl Send for GameLogger {}

impl std::fmt::Debug for GameLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameLogger")
            .field("verbosity", &self.verbosity)
            .field("capture_logs", &self.capture_logs)
            .field("log_count", &self.log_entries.borrow().len())
            .finish()
    }
}

impl Clone for GameLogger {
    fn clone(&self) -> Self {
        // Create a new logger with the same settings but empty log buffer
        // We don't clone the logs because they're arena-allocated
        GameLogger {
            verbosity: self.verbosity,
            step_header_printed: self.step_header_printed,
            numeric_choices: self.numeric_choices,
            output_format: self.output_format,
            bump: RefCell::new(Bump::new()),
            log_entries: RefCell::new(Vec::new()),
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
            bump: RefCell::new(Bump::new()),
            log_entries: RefCell::new(Vec::new()),
            capture_logs: data.capture_logs,
        })
    }
}

impl GameLogger {
    /// Create a new logger with default verbosity (Normal)
    pub fn new() -> Self {
        GameLogger {
            verbosity: VerbosityLevel::default(),
            step_header_printed: false,
            numeric_choices: false,
            output_format: OutputFormat::default(),
            bump: RefCell::new(Bump::new()),
            log_entries: RefCell::new(Vec::new()),
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
            bump: RefCell::new(Bump::new()),
            log_entries: RefCell::new(Vec::new()),
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

    /// Iterate over captured log entries (zero-copy access)
    ///
    /// Returns an iterator over log entries. This provides read-only access
    /// to the log buffer without any allocation or copying.
    ///
    /// # Example
    /// ```ignore
    /// for log in logger.logs() {
    ///     if log.message.contains("attack") {
    ///         println!("{}", log.message);
    ///     }
    /// }
    ///
    /// // Or count matching logs without copying:
    /// let attack_count = logger.logs()
    ///     .filter(|log| log.message.contains("attack"))
    ///     .count();
    /// ```
    pub fn logs(&self) -> impl Iterator<Item = &LogEntry<'_>> {
        // SAFETY: We cast from 'static back to the actual lifetime
        // This is safe because:
        // 1. The bump allocator owns the memory
        // 2. We never reset the bump allocator except in clear_logs()
        // 3. The returned iterator borrows self, preventing concurrent modification
        let entries = self.log_entries.borrow();
        let count = entries.len();
        (0..count).map(move |i| unsafe {
            let ptr = entries[i];
            &*ptr
        })
    }

    /// Clear the log buffer and reset the bump allocator
    pub fn clear_logs(&mut self) {
        self.log_entries.borrow_mut().clear();
        self.bump.borrow_mut().reset();
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

    /// Internal method to create and store a log entry in the bump allocator
    fn store_log_entry(&self, level: VerbosityLevel, message: &str, category: Option<&str>) {
        if !self.capture_logs {
            return;
        }

        let bump = self.bump.borrow();

        // Allocate strings in the bump arena
        let message_str: &str = bump.alloc_str(message);
        let category_str: Option<&str> = category.map(|c| {
            let s: &str = bump.alloc_str(c);
            s
        });

        // Create log entry in the bump arena
        let entry = bump.alloc(LogEntry {
            level,
            message: message_str,
            category: category_str,
        });

        // Store pointer to entry
        // SAFETY: We transmute the lifetime from 'bump to 'static
        // This is safe because we guarantee the bump allocator outlives the pointer
        let entry_ptr: *const LogEntry<'static> = unsafe {
            std::mem::transmute(entry as *const LogEntry<'_>)
        };

        self.log_entries.borrow_mut().push(entry_ptr);
    }

    /// Fast path for stdout logging without allocation
    #[inline]
    fn log_to_stdout(&self, level: VerbosityLevel, message: &str) {
        if level == VerbosityLevel::Minimal {
            println!("{}", message);
        } else {
            println!("  {}", message);
        }
    }

    /// Log at Silent level (always suppressed)
    #[inline]
    pub fn silent(&self, _message: &str) {
        // Silent messages are never printed or captured
    }

    /// Log at Minimal level (game outcomes, major events)
    #[inline]
    pub fn minimal(&self, message: &str) {
        // Early exit if message won't be used
        if VerbosityLevel::Minimal > self.verbosity && !self.capture_logs {
            return;
        }

        // Capture if enabled
        if self.capture_logs {
            self.store_log_entry(VerbosityLevel::Minimal, message, None);
        }

        // Output if verbosity allows
        if VerbosityLevel::Minimal <= self.verbosity {
            self.log_to_stdout(VerbosityLevel::Minimal, message);
        }
    }

    /// Log at Normal level (turns, steps, key actions)
    #[inline]
    pub fn normal(&self, message: &str) {
        // Early exit if message won't be used
        if VerbosityLevel::Normal > self.verbosity && !self.capture_logs {
            return;
        }

        // Capture if enabled
        if self.capture_logs {
            self.store_log_entry(VerbosityLevel::Normal, message, None);
        }

        // Output if verbosity allows
        if VerbosityLevel::Normal <= self.verbosity {
            self.log_to_stdout(VerbosityLevel::Normal, message);
        }
    }

    /// Log at Verbose level (all actions and state changes)
    #[inline]
    pub fn verbose(&self, message: &str) {
        // Early exit if message won't be used
        if VerbosityLevel::Verbose > self.verbosity && !self.capture_logs {
            return;
        }

        // Capture if enabled
        if self.capture_logs {
            self.store_log_entry(VerbosityLevel::Verbose, message, None);
        }

        // Output if verbosity allows
        if VerbosityLevel::Verbose <= self.verbosity {
            self.log_to_stdout(VerbosityLevel::Verbose, message);
        }
    }

    /// Log a controller decision at Normal level
    #[inline]
    pub fn controller_choice(&self, controller_name: &str, message: &str) {
        let should_log = self.numeric_choices || self.verbosity >= VerbosityLevel::Normal;

        if !should_log && !self.capture_logs {
            return;
        }

        // Build the formatted message using the bump allocator to avoid temporary allocation
        if self.capture_logs || should_log {
            // Allocate space for the formatted string in the bump arena
            let bump = self.bump.borrow();
            let mut formatted = bumpalo::collections::String::new_in(&*bump);
            write!(&mut formatted, ">>> {}: {}", controller_name, message).unwrap();
            let formatted_str = bump.alloc_str(&formatted);

            // Capture if enabled
            if self.capture_logs {
                let category_str = bump.alloc_str("controller_choice");
                let entry = bump.alloc(LogEntry {
                    level: VerbosityLevel::Normal,
                    message: formatted_str,
                    category: Some(category_str),
                });

                let entry_ptr: *const LogEntry<'static> = unsafe {
                    std::mem::transmute(entry as *const LogEntry<'_>)
                };
                self.log_entries.borrow_mut().push(entry_ptr);
            }

            // Output if should_log
            if should_log {
                println!("  {}", formatted_str);
            }
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
    fn test_log_capture() {
        let mut logger = GameLogger::new();
        logger.enable_capture();

        logger.normal("test message");
        logger.minimal("minimal message");

        let logs: Vec<_> = logger.logs().collect();
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
        let count = logger.logs()
            .filter(|log| log.message.contains("5"))
            .count();

        // Should match: 5, 15, 25, ..., 95, 50-59
        assert!(count > 10);
    }
}
