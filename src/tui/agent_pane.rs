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

use std::collections::VecDeque;
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
    pub id: Uuid,
    pub persona_name: String,
    pub connection: GrokConnection,
    pub messages: VecDeque<String>,
    pub input: String,
    pub scroll: u16,
    pub max_history: usize,
    pub is_waiting: bool,
    pub input_scroll: usize,
    pub input_max_lines: u16,

    pub chunk_receiver: mpsc::UnboundedReceiver<StreamChunk>,
    pub chunk_sender: mpsc::UnboundedSender<StreamChunk>,

    pub active_task: Option<tokio::task::JoinHandle<()>>,

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

        let (tx, rx) = mpsc::unbounded_channel();
        
        Self {
            id,
            persona_name: persona.name.clone(),
            connection: GrokConnection::new_without_output(persona),
            messages: VecDeque::new(),
            input: String::new(),
            scroll: 0,
            max_history: 1000,
            is_waiting: false,
            input_scroll: 0,
            input_max_lines: 20,
            chunk_sender: tx,
            chunk_receiver: rx,
            active_task: None,
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
        let msg = msg.into();
        self.messages.push_back(msg.clone());
        self.scroll_to_bottom();
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
        self.scroll = u16::MAX;
    }

    /// # wrap_input_text
    ///
    /// **Purpose:**
    /// Wraps the current input text to fit within the specified width.
    ///
    /// **Parameters:**
    /// - `width`: Maximum line width in characters
    ///
    /// **Returns:**
    /// Vector of wrapped lines
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    pub fn wrap_input_text(&self, width: usize) -> Vec<String> {
        if self.input.is_empty() {
            return vec![String::new()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in self.input.split_inclusive(|c: char| c.is_whitespace()) {
            if word.contains('\n') {
                let parts: Vec<&str> = word.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        lines.push(current_line.clone());
                        current_line.clear();
                    }
                    if !part.is_empty() {
                        let test_len = current_line.len() + part.len();
                        if test_len > width && !current_line.is_empty() {
                            lines.push(current_line.clone());
                            current_line = part.to_string();
                        } else {
                            current_line.push_str(part);
                        }
                    }
                }
                continue;
            }

            let test_len = current_line.len() + word.len();

            if test_len > width && !current_line.is_empty() {
                lines.push(current_line.trim_end().to_string());
                current_line = word.to_string();
            } else {
                current_line.push_str(word);
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        }
    }

    /// # scroll_input_to_bottom
    ///
    /// **Purpose:**
    /// Adjusts input scroll position to show the last lines of wrapped input text.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// None (mutates input_scroll state)
    pub fn scroll_input_to_bottom(&mut self) {
        let wrapped = self.wrap_input_text(100);
        self.input_scroll = wrapped.len().saturating_sub(self.input_max_lines as usize);
    }
}
