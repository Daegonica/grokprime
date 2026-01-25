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

use uuid::Uuid;
use crate::prelude::*;


/// # AgentPane
///
/// **Summary:**
/// Represents an individual agent conversation pane in the TUI with its own state.
///
/// **Fields:**
/// - `id`: Unique identifier for this agent pane
/// - `persona_name`: The persona/agent name displayed in the UI
/// - `connection`: GrokConnection instance for API communication
/// - `messages`: Message history for this agent
/// - `input`: Current input text for this agent (currently unused - ShadowApp.input is used)
/// - `scroll`: Vertical scroll position in message history
/// - `max_history`: Maximum number of messages to retain
/// - `is_waiting`: Whether the agent is waiting for a response
/// - `input_scroll`: Vertical scroll position in input area
/// - `input_max_lines`: Maximum visible lines in input area
/// - `chunk_receiver`: Receives StreamChunk messages from async streaming task
/// - `chunk_sender`: Sends StreamChunk messages to this pane
/// - `active_task`: Handle to the currently running async response task
/// - `thinking_animation_frame`: Current frame of the thinking animation (0-3)
///
/// **Usage Example:**
/// ```rust
/// let persona_ref = Arc::new(persona);
/// let pane = AgentPane::new(Uuid::new_v4(), persona_ref);
/// pane.add_message("Welcome!");
/// ```
#[derive(Debug)]
pub struct AgentPane {
    pub agent: AgentInfo,

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
    /// Creates a new agent pane with the specified ID and persona configuration.
    ///
    /// **Parameters:**
    /// - `id`: Unique identifier for this agent
    /// - `persona`: Arc-wrapped persona configuration
    ///
    /// **Returns:**
    /// Initialized AgentPane with default values and communication channels
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    pub fn new(id: Uuid, persona: PersonaRef) -> Self {

        Self {
            agent: AgentInfo::new(id, persona),
            scroll: 0,
            auto_scroll: true,
            input_scroll: 0,
            input_max_lines: 20,
            thinking_animation_frame: 0,
         }
    }

    /// # add_message
    ///
    /// **Purpose:**
    /// Adds a message to this agent's message history and scrolls to bottom.
    ///
    /// **Parameters:**
    /// - `msg`: The message content (anything that converts to String)
    ///
    /// **Returns:**
    /// None (mutates internal state)
    pub fn add_message(&mut self, msg: impl Into<String>) {
        
        self.agent.add_message(msg);

        if self.auto_scroll {
            self.scroll_to_bottom();
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
