//! # Daegonica Module: utilities::cli
//!
//! **Purpose:** Command-line argument parsing using clap
//!
//! **Context:**
//! - Determines application run mode (TUI vs CLI)
//! - Parsed at application startup in main.rs
//!
//! **Responsibilities:**
//! - Define CLI argument structure
//! - Parse command-line flags
//! - Provide mode detection helper
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use clap::Parser;

/// # Args
///
/// **Summary:**
/// Command-line arguments for controlling application mode.
///
/// **Fields:**
/// - `tui`: Enable TUI mode (default: true)
/// - `cli`: Enable CLI mode (conflicts with tui)
///
/// **Usage Example:**
/// ```rust
/// let args = Args::parse();
/// if args.is_tui_mode() {
///     run_tui_mode().await?;
/// }
/// ```
#[derive(Parser, Debug)]
#[command(name = "grokprime-brain")]
#[command(about = "Shadow AI Assistant", long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = true)]
    pub tui: bool,

    #[arg(long, conflicts_with = "tui")]
    pub cli: bool,

    #[arg(long, default_value = "shadow")]
    pub persona: String,
}

impl Args {
    /// # is_tui_mode
    ///
    /// **Purpose:**
    /// Determines if the application should run in TUI mode.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// `bool` - true for TUI mode, false for CLI mode
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    ///
    /// **Examples:**
    /// ```rust
    /// if args.is_tui_mode() {
    ///     // Run with terminal UI
    /// }
    /// ```
    pub fn is_tui_mode(&self) -> bool {
        !self.cli
    }
}