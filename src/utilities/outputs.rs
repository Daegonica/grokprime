//! # Daegonica Module: utilities::outputs
//!
//! **Purpose:** Output abstraction trait for CLI and TUI display modes
//!
//! **Context:**
//! - Provides unified interface for displaying messages
//! - Enables switching between CLI println and TUI message buffers
//! - Used throughout the application for all output
//!
//! **Responsibilities:**
//! - Define OutputHandler trait for message display
//! - Implement CLI output via println
//! - Implement TUI output via shared message buffer
//! - Provide SharedOutput type alias for thread-safe sharing
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use std::sync::{Arc, Mutex};

/// # OutputHandler
///
/// **Summary:**
/// Trait for abstracting message output across different display modes.
///
/// **Methods:**
/// - `display`: Display a message string using the implementation's output mechanism
///
/// **Usage Example:**
/// ```rust
/// let output: SharedOutput = Arc::new(CliOutput);
/// output.display("Hello!".to_string());
/// ```
pub trait OutputHandler: Send {
    fn display(&self, msg: String);
}

/// # CliOutput
///
/// **Summary:**
/// Simple CLI output implementation that prints to stdout.
///
/// **Usage Example:**
/// ```rust
/// let output = CliOutput;
/// output.display("Message".to_string());
/// ```
pub struct CliOutput;

impl OutputHandler for CliOutput {
    fn display(&self, msg: String) {
        println!("{}", msg);
    }
}

/// # TuiOutput
///
/// **Summary:**
/// TUI output implementation that accumulates messages in a shared buffer.
///
/// **Fields:**
/// - `messages`: Thread-safe buffer for accumulating messages
///
/// **Usage Example:**
/// ```rust
/// let buffer = Arc::new(Mutex::new(Vec::new()));
/// let output = TuiOutput::new(buffer);
/// output.display("Message".to_string());
/// ```
pub struct TuiOutput {
    messages: Arc<Mutex<Vec<String>>>,
}

impl TuiOutput {
    /// # new
    ///
    /// **Purpose:**
    /// Creates a new TuiOutput with the specified message buffer.
    ///
    /// **Parameters:**
    /// - `messages`: Shared message buffer for accumulating output
    ///
    /// **Returns:**
    /// Initialized TuiOutput instance
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    pub fn new(messages: Arc<Mutex<Vec<String>>>) -> Self {
        Self { messages }
    }
}

impl OutputHandler for TuiOutput {
    fn display(&self, msg: String) {
        if let Ok(mut msgs) = self.messages.lock() {
            msgs.push(msg);
        }
    }
}

/// # SharedOutput
///
/// **Summary:**
/// Type alias for thread-safe, shareable OutputHandler trait objects.
///
/// **Usage Example:**
/// ```rust
/// let output: SharedOutput = Arc::new(CliOutput);
/// let output_clone = Arc::clone(&output);
/// ```
pub type SharedOutput = Arc<dyn OutputHandler>;