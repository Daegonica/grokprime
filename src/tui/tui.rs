//! # Daegonica Module: tui::tui
//!
//! **Purpose:** Core TUI application state and rendering logic
//!
//! **Context:**
//! - Main TUI implementation using ratatui framework
//! - Manages multiple agent panes and message display
//! - Handles user input and keyboard events
//!
//! **Responsibilities:**
//! - Render TUI layout with message history and input areas
//! - Handle keyboard input and commands
//! - Manage multiple agent panes with tab switching
//! - Display unified and per-agent message streams
//! - Text wrapping and scrolling for messages and input
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use ratatui::{
    // backend::Backend,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Position},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
    prelude::Rect,
};
use std::time::SystemTime;
use std::collections::VecDeque;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::prelude::*;

/// # UnifiedMessage
///
/// **Summary:**
/// Represents a message with source tracking and timestamp for unified display.
///
/// **Fields:**
/// - `text`: The message content
/// - `source`: Where the message originated (Global or specific Agent)
/// - `timestamp`: When the message was created
///
/// **Usage Example:**
/// ```rust
/// let msg = UnifiedMessage {
///     text: "Hello".to_string(),
///     source: MessageSource::Global,
///     timestamp: SystemTime::now(),
/// };
/// ```
#[derive(Debug)]
pub struct UnifiedMessage {
    pub text: String,
    pub source: MessageSource,
    pub timestamp: SystemTime,
}

/// # MessageSource
///
/// **Summary:**
/// Indicates where a message originated from in the TUI.
///
/// **Variants:**
/// - `Global`: Message displayed globally across all panes
/// - `Agent(String)`: Message from a specific agent with persona name
#[derive(Debug)]
pub enum MessageSource {
    Global,
    Agent(String),
}

/// # AgentPane
///
/// **Summary:**
/// Represents an individual agent conversation pane in the TUI with its own state.
///
/// **Fields:**
/// - `id`: Unique identifier for this agent pane
/// - `persona_name`: The persona/agent name displayed in the UI
/// - `messages`: Message history for this agent
/// - `input`: Current input text for this agent
/// - `scroll`: Vertical scroll position in message history
/// - `max_history`: Maximum number of messages to retain
/// - `is_waiting`: Whether the agent is waiting for a response
/// - `input_scroll`: Vertical scroll position in input area
/// - `input_max_lines`: Maximum visible lines in input area
/// - `pending_messages`: Thread-safe buffer for messages from async operations
///
/// **Usage Example:**
/// ```rust
/// let pane = AgentPane::new(Uuid::new_v4(), "shadow".to_string());
/// pane.add_message("Welcome!");
/// ```
#[derive(Debug)]
pub struct AgentPane {
    pub id: Uuid,
    pub persona_name: String,
    pub messages: VecDeque<String>,
    pub input: String,
    pub scroll: u16,
    pub max_history: usize,
    pub is_waiting: bool,
    pub input_scroll: usize,
    pub input_max_lines: u16,
    pub pending_messages: Arc<Mutex<Vec<String>>>,
}

impl AgentPane {
    /// # new
    ///
    /// **Purpose:**
    /// Creates a new agent pane with the specified ID and persona name.
    ///
    /// **Parameters:**
    /// - `id`: Unique identifier for this agent
    /// - `persona_name`: Display name for the agent persona
    ///
    /// **Returns:**
    /// Initialized AgentPane with default values
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    pub fn new(id: Uuid, persona_name: String) -> Self {
         Self {
            id,
            persona_name,
            messages: VecDeque::new(),
            input: String::new(),
            scroll: 0,
            max_history: 1000,
            is_waiting: false,
            input_scroll: 0,
            input_max_lines: 20,
            pending_messages: Arc::new(Mutex::new(Vec::new())),
         }
    }

    /// # get_message_buffer
    ///
    /// **Purpose:**
    /// Returns a clone of the shared message buffer for async message accumulation.
    ///
    /// **Returns:**
    /// Arc-wrapped Mutex-protected vector of pending messages
    pub fn get_message_buffer(&self) -> Arc<Mutex<Vec<String>>> {
        Arc::clone(&self.pending_messages)
    }

    /// # flush_pending_messages
    ///
    /// **Purpose:**
    /// Moves all pending messages from the buffer into the main message history.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// None (mutates internal state)
    pub fn flush_pending_messages(&mut self) {
        let messages_to_add: Vec<String> = if let Ok(mut pending) = self.pending_messages.lock() {
            pending.drain(..).collect()
        } else {
            Vec::new()
        };
        for msg in messages_to_add {
            self.add_message(msg);
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

/// # ShadowApp
///
/// **Summary:**
/// Main TUI application state managing multiple agent panes and global messages.
///
/// **Fields:**
/// - `messages`: Global message history displayed across all panes
/// - `input`: Current input text in the active pane
/// - `scroll`: Global scroll position
/// - `max_history`: Maximum messages to retain in history
/// - `user_input`: Optional user input handler
/// - `pending_messages`: Thread-safe buffer for async message accumulation
/// - `is_waiting`: Whether the app is waiting for a response
/// - `input_scroll`: Scroll position in input area
/// - `input_max_lines`: Maximum visible lines in input
/// - `agents`: Map of agent IDs to their panes
/// - `agent_order`: Ordered list of agent IDs for tab switching
/// - `current_agent`: Currently selected agent ID
/// - `unified_messages`: All messages with source tracking
///
/// **Usage Example:**
/// ```rust
/// let mut app = ShadowApp::new();
/// app.add_agent(Uuid::new_v4(), "shadow".to_string());
/// ```
#[derive(Debug)]
pub struct ShadowApp {
    pub messages: VecDeque<String>,
    pub input: String,
    pub scroll: u16,
    pub max_history: usize,
    pub user_input: Option<UserInput>,
    pub pending_messages: Arc<Mutex<Vec<String>>>,
    pub is_waiting: bool,
    pub input_scroll: usize,
    pub input_max_lines: u16,
    pub agents: HashMap<Uuid, AgentPane>,
    pub agent_order: Vec<Uuid>,
    pub current_agent: Option<Uuid>,
    pub unified_messages: VecDeque<UnifiedMessage>,
}

impl Default for ShadowApp {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
            input: String::new(),
            scroll: 0,
            max_history: 1000,
            user_input: None,
            pending_messages: Arc::new(Mutex::new(Vec::new())),
            is_waiting: false,
            input_scroll: 0,
            input_max_lines: 20,
            agents: HashMap::new(),
            agent_order: Vec::new(),
            current_agent: None,
            unified_messages: VecDeque::new(),
        }
    }
}

impl ShadowApp {
    /// # new
    ///
    /// **Purpose:**
    /// Creates a new ShadowApp instance with default values.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// Initialized ShadowApp
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    pub fn new() -> Self {
        Self::default()
    }

    /// # add_agent
    ///
    /// **Purpose:**
    /// Adds a new agent pane to the application and makes it the current agent.
    ///
    /// **Parameters:**
    /// - `id`: Unique identifier for the new agent
    /// - `persona_name`: Display name for the agent persona
    ///
    /// **Returns:**
    /// None (mutates internal state)
    pub fn add_agent(&mut self, id: Uuid, persona_name: String) {
        let pane = AgentPane::new(id, persona_name);
        self.agent_order.push(id);
        self.current_agent = Some(id);
        self.agents.insert(id, pane);
    }

    /// # remove_agent
    ///
    /// **Purpose:**
    /// Removes an agent pane from the application and adjusts the current selection.
    ///
    /// **Parameters:**
    /// - `id`: The agent ID to remove
    ///
    /// **Returns:**
    /// None (mutates internal state)
    pub fn remove_agent(&mut self, id: Uuid) {
        self.agents.remove(&id);
        self.agent_order.retain(|&x| x != id);
        if self.current_agent == Some(id) {
            self.current_agent = self.agent_order.last().cloned();
        }
    }

    /// # switch_agent
    ///
    /// **Purpose:**
    /// Switches to the next or previous agent in the tab order.
    ///
    /// **Parameters:**
    /// - `next`: true for next agent, false for previous
    ///
    /// **Returns:**
    /// None (mutates current_agent)
    pub fn switch_agent(&mut self, next: bool) {
        if self.agent_order.is_empty() {return;}
        if let Some(current) = self.current_agent {
            let idx = self.agent_order.iter().position(|&x| x == current).unwrap_or(0);
            let new_idx = if next {
                (idx +1) % self.agent_order.len()
            } else {
                (idx + self.agent_order.len() -1) % self.agent_order.len()
            };
            self.current_agent = Some(self.agent_order[new_idx]);
        } else {
            self.current_agent = self.agent_order.first().cloned();
        }
    }

    /// # current_pane_mut
    ///
    /// **Purpose:**
    /// Returns a mutable reference to the currently selected agent pane.
    ///
    /// **Returns:**
    /// Option containing mutable reference to AgentPane, or None if no current agent
    pub fn current_pane_mut(&mut self) -> Option<&mut AgentPane> {
        self.current_agent.and_then(move |id| self.agents.get_mut(&id))
    }

    /// # get_message_buffer
    ///
    /// **Purpose:**
    /// Returns a clone of the global message buffer for async message accumulation.
    ///
    /// **Returns:**
    /// Arc-wrapped Mutex-protected vector of pending messages
    pub fn get_message_buffer(&self) -> Arc<Mutex<Vec<String>>> {
        Arc::clone(&self.pending_messages)
    }

    /// # flush_pending_messages
    ///
    /// **Purpose:**
    /// Moves all pending global messages from the buffer into the main message history.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// None (mutates internal state)
    pub fn flush_pending_messages(&mut self) {
        let messages_to_add: Vec<String> = if let Ok(mut pending) = self.pending_messages.lock() {
            pending.drain(..).collect()
        } else {
            Vec::new()
        };

        for msg in messages_to_add {
            self.add_message(msg);
        }
    }
    
    /// # add_message
    ///
    /// **Purpose:**
    /// Adds a global message to both the legacy and unified message queues.
    ///
    /// **Parameters:**
    /// - `msg`: The message content (anything that converts to String)
    ///
    /// **Returns:**
    /// None (mutates internal state)
    pub fn add_message(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.messages.push_back(msg.clone());
        
        self.unified_messages.push_back(UnifiedMessage {
            text: msg,
            source: MessageSource::Global,
            timestamp: SystemTime::now(),
        });

        self.scroll_to_bottom();
    }

    /// # scroll_to_bottom
    ///
    /// **Purpose:**
    /// Sets global scroll position to maximum to show the most recent messages.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// None (mutates scroll state)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll = u16::MAX;
    }

    fn scroll_input_to_bottom(&mut self) {
        let wrapped = self.wrap_input_text(100);
        self.input_scroll = wrapped.len().saturating_sub(self.input_max_lines as usize);
    }

    /// # handle_key
    ///
    /// **Purpose:**
    /// Processes keyboard input events and updates application state accordingly.
    ///
    /// **Parameters:**
    /// - `key`: The keyboard event to process
    ///
    /// **Returns:**
    /// `bool` - true to continue running, false to exit
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    ///
    /// **Examples:**
    /// ```rust
    /// let should_continue = app.handle_key(key);
    /// if !should_continue { break; }
    /// ```
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            
            KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.switch_agent(true);
                true
            }

            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.switch_agent(false);
                true
            }

            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(id) = self.current_agent {
                    self.remove_agent(id);
                }
                true
            }

            // Input Text control
            KeyCode::Char(c) => {
                self.input.push(c);
                self.scroll_input_to_bottom();
                true
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.scroll_input_to_bottom();
                true
            }
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    let line = self.input.trim().to_string();
                    let user_input = self.user_input.clone();

                    if line == "status" {

                        let mut status = String::new();
                        status.push_str(&format!("Current agent: {}\n", self.current_agent
                            .and_then(|id| self.agents.get(&id))
                            .map(|pane| &pane.persona_name)
                            .unwrap_or(&"<none>".to_string())));

                        status.push_str(&format!(" - Current pane: {}\n", self.current_pane_mut()
                        .map(|pane| &pane.persona_name)
                        .unwrap_or(&"<none>".to_string())));

                        status.push_str(" - All agents:\n");
                        
                        for id in &self.agent_order {
                            let pane = &self.agents[id];
                            let marker = if Some(*id) == self.current_agent {" ->"} else {" "};
                            status.push_str(&format!("{} {}\n", marker, pane.persona_name));
                        }
                        status.push_str(&format!(" - Total tabs: {}", self.agent_order.len()));

                        if let Some(pane) = self.current_pane_mut() {
                            pane.add_message(status);
                        } else {
                            self.add_message(status);
                        }

                        self.input.clear();
                    }

                    let mut agent_unified_message: Option<UnifiedMessage> = None;
                    if let Some(pane) = self.current_pane_mut() {
                        if let Some(user_input) = user_input {
                            match user_input.process_input(&line) {
                                InputAction::DoNothing => {
                                    self.input.clear();
                                }
                                InputAction::ContinueNoSend(msg) => {
                                    self.add_message(format!("> {}", msg));
                                    self.input.clear();
                                }
                                InputAction::Quit => {
                                    self.input.clear();
                                    return false;
                                },
                                InputAction::SendAsMessage(content) => {
                                    pane.add_message(format!("> {}", content));
                                    self.input.clear();
                                }
                                InputAction::PostTweet(_content) => todo!(),
                                InputAction::DraftTweet(_content) => todo!(),

                                InputAction::NewAgent(persona) => {
                                    let id = Uuid::new_v4();
                                    self.add_agent(id, persona.clone());
                                    self.current_agent = Some(id);
                                    self.add_message(format!("Created new agent with persona '{}'", persona));
                                    self.input.clear();
                                }
                                InputAction::CloseAgent => {
                                    if let Some(id) = self.current_agent {
                                        self.remove_agent(id);
                                        self.add_message("Closed current agent.");
                                    }
                                    self.input.clear();
                                }
                                InputAction::ListAgents => {
                                    let personas = vec!["shadow"];
                                    self.add_message(format!("Available personas: {}", personas.join(", ")));
                                    self.input.clear();
                                }

                            }
                        } else {
                            pane.add_message(format!("> {}", line));
                            agent_unified_message = Some(UnifiedMessage {
                                text: line.clone(),
                                source: MessageSource::Agent(pane.persona_name.clone()),
                                timestamp: SystemTime::now(),
                            });
                            self.input.clear();
                        }
                    } else {
                        self.add_message("No agent available. Create one with 'new <persona>'");
                        self.input.clear();
                    }
                    if let Some(msg) = agent_unified_message {
                        self.unified_messages.push_back(msg);
                    }
                }
                true
            }

            // Input Scroll control
            KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input_scroll = self.input_scroll.saturating_sub(1);
                true
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let wrapped = self.wrap_input_text(100);
                let max_scroll = wrapped.len().saturating_sub(self.input_max_lines as usize);
                self.input_scroll = (self.input_scroll + 1).min(max_scroll);
                true
            }

            // History Scroll control
            KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
                true
            }
            KeyCode::Down => {
                self.scroll = self.scroll.saturating_add(1);
                true
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
                true
            }
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_add(10);
                true
            }
            KeyCode::Esc => {
                return false;
            }
            _ => true,
        }
    }

    fn calculate_input_height(&self, width: u16) -> u16 {
        if self.is_waiting{
            return 3;
        }

        let available_width = width.saturating_sub(6) as usize;
        if available_width == 0{
            return 3;
        }

        let lines_needed = if self.input.is_empty() {
            1
        } else {
            (self.input.len() / available_width) + 1
        };

        (lines_needed.min(self.input_max_lines as usize) as u16) + 2
    }

    // Need to take out all the basic code that can be turned into functions for easier reading.
    fn unified_messages(&self) -> Vec<Line<'_>> {
        let mut lines: Vec<Line> = Vec::new();
        for unified in &self.unified_messages {
            let content = if unified.text.starts_with('>') {
                Line::from(Span::styled(
                    unified.text.clone(),
                    Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(unified.text.clone())
            };
            lines.push(content);
        }
        lines
    }

    fn current_pane(&self) -> Option<&AgentPane> {
        self.current_agent.and_then(|id| self.agents.get(&id))
    }

    fn pan_messages(&self) -> Vec<Line<'_>> {
        let mut lines: Vec<Line> = Vec::new();
        if let Some(pane) = self.current_pane() {
            for msg in &pane.messages {
                let content = if msg.starts_with('>') {
                    Line::from(Span::styled(
                        msg.clone(),
                        Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(msg.clone())
                };
                lines.push(content);
            }
        }
        lines
    }

    fn render_input(&self, frame: &mut Frame<'_>, area: Rect) {
        let input_text = if self.is_waiting {
            Text::from(vec![
                Line::from(vec![
                    Span::styled(" > ", Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
                    Span::styled("Shadow is thinking...", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                ])
            ])
        } else {
            let available_width = area.width.saturating_sub(6) as usize;

            let wrapped_lines = self.wrap_input_text(available_width);
            let total_lines = wrapped_lines.len();

            let max_visible = (area.height.saturating_sub(2)) as usize;
            let scroll_offset = self.input_scroll.min(total_lines.saturating_sub(max_visible));

            let visible_lines: Vec<Line> = wrapped_lines
                .iter()
                .skip(scroll_offset)
                .take(max_visible)
                .enumerate()
                .map(|(idx, line)| {
                    if idx == 0 {
                        Line::from(vec![
                            Span::styled(" > ", Style::default().fg(Color::Rgb(255, 140, 0))),
                            Span::raw(line.to_string()),
                        ])
                    } else {
                        Line::from(format!("   {}", line))
                    }
                })
                .collect();

            Text::from(visible_lines)
        };

        let input_widget = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 140, 0)))
                    .title(" Input "),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(input_widget, area);
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {

        let input_height = self.calculate_input_height(frame.area().width);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(input_height),
            ])
            .split(frame.area());
        let message_area = chunks[0];
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(message_area);

        // Gather messages from each pane and unified messages
        let pane_lines = self.pan_messages();
        let unified_lines = self.unified_messages();
        let mut global_scroll = self.scroll;
        let mut agent_scroll = self.current_pane()
                .map(|p| p.scroll)
                .unwrap_or(0);

        render_message_section(
            frame,
            split[1],
            unified_lines,
            "Global",
            &mut global_scroll,
        );

        render_message_section(
            frame,
            split[0],
            pane_lines,
            "Agent",
            &mut agent_scroll,
        );

        if let Some(pane) = self.current_pane_mut() {
            pane.scroll = agent_scroll;
        }

        // Setup input area
        let input_area = chunks[1];

        self.render_input(frame, input_area);


        if input_area.height > 2 && input_area.width > 6 && !self.is_waiting {
            let width = input_area.width.saturating_sub(6) as usize;
            let wrapped = self.wrap_input_text(width);

            let mut chars_counted = 0;
            let mut cursor_line = 0;
            let mut cursor_col_in_line = 0;

            for (line_idx, line) in wrapped.iter().enumerate() {
                let line_len = line.len();
                if chars_counted + line_len >= self.input.len() {
                    cursor_line = line_idx;
                    cursor_col_in_line = self.input.len() - chars_counted;
                    break;
                }
                chars_counted += line_len;
            }

            if cursor_line >= self.input_scroll {
                let visible_line = cursor_line - self.input_scroll;
                let max_visible = input_area.height.saturating_sub(2) as usize;

                if visible_line < max_visible {
                    let cursor_pos = Position {
                        x: input_area.x + 3 + cursor_col_in_line as u16,
                        y: input_area.y + 1 + visible_line as u16,
                    };
                    frame.set_cursor_position(cursor_pos);
                }
            }
        }
    }

    /// # wrap_input_text
    ///
    /// **Purpose:**
    /// Wraps the current input text to fit within the specified width (internal helper).
    ///
    /// **Parameters:**
    /// - `width`: Maximum line width in characters
    ///
    /// **Returns:**
    /// Vector of wrapped lines
    fn wrap_input_text(&self, width: usize) -> Vec<String> {
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
}
fn render_message_section(
    frame: &mut Frame,
    area: Rect,
    lines: Vec<Line>,
    title: &str,
    scroll: &mut u16,
) {
    // Calculate content height and visible height
    let content_height = lines.len() as u16;
    let visible_height = area.height.saturating_sub(2);
    let content_len = content_height as usize;
    let viewport_len = visible_height as usize;

    // Set scroll within bounds
    let mut max_scroll = content_height.saturating_sub(visible_height);
    *scroll = *scroll.min(&mut max_scroll);
    if *scroll == u16::MAX || *scroll > max_scroll {
        *scroll = max_scroll;
    }
    let mut scrollbar_state = ScrollbarState::default()
        .content_length(content_len)
        .viewport_content_length(viewport_len)
        .position(*scroll as usize);
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("█");

    // Add all messages to 1 'text' for display
    let text = Text::from(lines.clone());
    // Set border and title styles
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(255, 140, 0)))
                .title_style(Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)),
        )
        .wrap(Wrap { trim: true })
        .scroll((*scroll, 0));

    // Render message area
    frame.render_widget(paragraph, area);
    // Add scrollbar to message area
    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}
