//! # Daegonica Module: tui::agent_pane
//!
//! **Purpose:** Individual agent conversation pane state and behavior
//!
//! **Context:**
//! - Represents a single agent tab in the TUI
//! - Manages conversation messages and streaming responses
//! - Handles channel communication for async operations
//!
//! **Responsibilities:**
//! - Maintain agent-specific message history
//! - Manage GrokConnection for this agent
//! - Handle streaming response channels
//! - Provide scrolling and text wrapping utilities
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------


/// # AgentPane
///
/// **Summary:**
/// Represents UI state for an individual agent conversation pane in the TUI.
///
/// **Fields:**
/// - `scroll`: Vertical scroll position in message history
/// - `auto_scroll`: Whether to automatically scroll to bottom on new messages
/// - `input_scroll`: Vertical scroll position in input area
/// - `input_max_lines`: Maximum visible lines in input area
/// - `thinking_animation_frame`: Current frame of the thinking animation (0-3)
///
/// **Design Note:**
/// AgentPane only contains UI state. Agent business logic (messages, connection, etc.)
/// is stored in AgentManager. This separation prevents state duplication bugs.
///
/// **Usage Example:**
/// ```rust
/// let pane = AgentPane::new();
/// pane.scroll_to_bottom();
/// ```
#[derive(Debug)]
pub struct AgentPane {
    pub scroll: u16,
    pub auto_scroll: bool,
    pub input_scroll: usize,
    pub input_max_lines: u16,
    pub thinking_animation_frame: usize,
}

impl AgentPane {
    /// # new
    ///
    /// **Purpose:**
    /// Creates a new agent pane with default UI state.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// Initialized AgentPane with default UI values
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    pub fn new() -> Self {
        Self {
            scroll: 0,
            auto_scroll: true,
            input_scroll: 0,
            input_max_lines: 20,
            thinking_animation_frame: 0,
         }
    }



    /// # scroll_to_bottom
    ///
    /// **Purpose:**
    /// Sets scroll position to maximum to show the most recent messages.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// None (mutates scroll state)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll = u16::MAX;  // Will be clamped to actual max by render
        self.auto_scroll = true;   // Re-enable auto-scroll
    }
}
