//! # Daegonica Module: utilities
//!
//! **Purpose:** Common utilities and helper modules
//!
//! **Context:**
//! - Provides output abstraction for TUI and CLI modes
//! - Handles command-line argument parsing
//!
//! **Responsibilities:**
//! - Expose CLI and output modules
//! - Re-export commonly used types
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

pub mod cli;
pub mod outputs;

pub use cli::*;
pub use outputs::*;