//! # Daegonica Module: tui
//!
//! **Purpose:** Terminal User Interface implementation
//!
//! **Context:**
//! - Provides interactive TUI mode for the application
//! - Uses ratatui for rendering and crossterm for event handling
//!
//! **Responsibilities:**
//! - Coordinate agent panes, message display, and user input
//! - Render beautiful terminal interface with split panes
//! - Handle keyboard input and command routing
//! - Manage async task communication via channels
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

// Module declarations
pub mod agent_pane;
pub mod app;
pub mod command_handler;
pub mod widgets;

// Re-exports for public API
pub use app::{ShadowApp, MessageSource, UnifiedMessage};
pub use agent_pane::AgentPane;