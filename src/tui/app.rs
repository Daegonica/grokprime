//! # Daegonica Module: tui::app
//!
//! **Purpose:** Core TUI application state and coordination
//!
//! **Context:**
//! - Central state manager for the entire TUI application
//! - Coordinates multiple agent panes and global messages
//! - Handles keyboard input routing and event processing
//!
//! **Responsibilities:**
//! - Manage agent lifecycle (creation, removal, switching)
//! - Coordinate message flow between agents and UI
//! - Handle keyboard input and route to command handlers
//! - Orchestrate rendering pipeline
//! - Maintain global application state
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use std::collections::{HashMap, VecDeque};
use std::time::SystemTime;
use uuid::Uuid;
use std::path::Path;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Text, Line, Span},
    Frame,
    widgets::{Block, Borders, Paragraph},
};

use crate::prelude::*;
use crate::tui::agent_pane::AgentPane;
use crate::tui::widgets::render_message_section;
use crate::commands::{from_input_action, CommandResult};

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
/// - `is_waiting`: Whether the app is waiting for a response
/// - `input_scroll`: Scroll position in input area
/// - `input_max_lines`: Maximum visible lines in input
/// - `personas`: Map of persona names to their configurations
/// - `agents`: Map of agent IDs to their panes
/// - `agent_order`: Ordered list of agent IDs for tab switching
/// - `current_agent`: Currently selected agent ID
/// - `unified_messages`: All messages with source tracking
///
/// **Usage Example:**
/// ```rust
/// let mut app = ShadowApp::new();
/// let persona_ref = Arc::new(persona);
/// app.add_agent(Uuid::new_v4(), persona_ref);
/// ```
#[derive(Debug)]
pub struct ShadowApp {
    pub messages: VecDeque<String>,
    pub input: String,
    pub scroll: u16,
    pub max_history: usize,
    pub user_input: Option<UserInput>,
    pub is_waiting: bool,
    pub input_scroll: usize,
    pub input_max_lines: u16,
    pub personas: HashMap<String, PersonaRef>,
    pub agents: HashMap<Uuid, AgentPane>,
    pub agent_order: Vec<Uuid>,
    pub current_agent: Option<Uuid>,
    pub unified_messages: VecDeque<UnifiedMessage>,
}

impl Default for ShadowApp {
    fn default() -> Self {
        let tui_config = &GLOBAL_CONFIG.tui;
        Self {
            messages: VecDeque::new(),
            input: String::new(),
            scroll: 0,
            max_history: tui_config.max_history_size,
            user_input: None,
            is_waiting: false,
            input_scroll: 0,
            input_max_lines: tui_config.max_input_lines,
            personas: HashMap::new(),
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

    /// # load_personas
    ///
    /// **Purpose:**
    /// Loads persona configurations from YAML files and stores them in the app.
    ///
    /// **Parameters:**
    /// - `persona_paths`: Vector of paths to persona YAML files
    ///
    /// **Returns:**
    /// `anyhow::Result<()>` - Success or error if any persona fails to load
    ///
    /// **Errors / Failures:**
    /// - File not found
    /// - Invalid YAML format
    /// - Missing required fields in persona config
    ///
    /// **Examples:**
    /// ```rust
    /// let paths = vec![Path::new("personas/shadow/shadow.yaml")];
    /// app.load_personas(paths)?;
    /// ```
    pub fn load_personas(&mut self, persona_paths: Vec<&Path>) -> anyhow::Result<()> {
        for path in persona_paths {
            let persona = Persona::from_yaml_file(path)?;
            self.personas.insert(persona.name.clone(), Arc::new(persona));
        }
        Ok(())
    }

    /// # add_agent
    ///
    /// **Purpose:**
    /// Adds a new agent pane to the application and makes it the current agent.
    ///
    /// **Parameters:**
    /// - `id`: Unique identifier for the new agent
    /// - `persona`: Arc-wrapped persona configuration
    ///
    /// **Returns:**
    /// None (mutates internal state)
    pub fn add_agent(&mut self, id: Uuid, persona: PersonaRef) {
        let pane = AgentPane::new(id, persona);
        self.agent_order.push(id);
        self.current_agent = Some(id);
        self.agents.insert(id, pane);
    }

    /// # get_agent_name
    ///
    /// **Purpose:**
    /// Retrieves the persona name for a given agent ID.
    ///
    /// **Parameters:**
    /// - `id`: The agent UUID to look up
    ///
    /// **Returns:**
    /// String containing the persona name, or "<unknown>" if not found
    fn get_agent_name(&self, id: Uuid) -> String {
        self.agents.get(&id)
            .map(|pane| pane.persona_name.clone())
            .unwrap_or("<unknown>".to_string())
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
        if let Some(pane) = self.agents.get_mut(&id) {
            if let Some(task) = pane.active_task.take() {
                task.abort();
            }
            // Channel will be automatically dropped when pane is removed
        }

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

    /// # current_pane
    ///
    /// **Purpose:**
    /// Returns an immutable reference to the currently selected agent pane.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// `Option<&AgentPane>` - Reference to current pane, or None if no agent selected
    pub fn current_pane(&self) -> Option<&AgentPane> {
        self.current_agent.and_then(|id| self.agents.get(&id))
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
    
    /// # poll_channels
    ///
    /// **Purpose:**
    /// Polls all agent channels for incoming StreamChunk messages and updates pane state.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// None (mutates agent pane states)
    ///
    /// **Details:**
    /// - Processes Delta chunks by appending to last message
    /// - Handles Complete chunks by updating connection state
    /// - Processes Error chunks by displaying error messages
    /// - Updates thinking animation frames while waiting
    pub fn poll_channels(&mut self) {
        for (_agent_id, pane) in self.agents.iter_mut() {
            if pane.is_waiting {
                pane.thinking_animation_frame = (pane.thinking_animation_frame + 1) % 4;
            }
            
            // Poll the receiver without taking it
            while let Ok(chunk) = pane.chunk_receiver.try_recv() {
                match chunk {
                    StreamChunk::Delta(text) => {
                        
                        if let Some(last_msg) = pane.messages.back_mut() {
                            if !last_msg.starts_with('>') {
                                last_msg.push_str(&text);
                            } else {
                                pane.add_message(text);
                            }
                        } else {
                            pane.add_message(text);
                        }
                        
                        if pane.auto_scroll {
                            pane.scroll_to_bottom();
                        }
                    }

                    StreamChunk::Complete{response_id, full_reply} => {
                        pane.connection.set_last_response_id(response_id.clone());

                        pane.connection.conversation.local_history.push(Message {
                            role: "assistant".to_string(),
                            content: full_reply,
                        });

                        pane.is_waiting = false;
                        pane.active_task = None;
                    }

                    StreamChunk::Error(err) => {
                        pane.add_message(format!("Error: {}", err));
                        pane.add_message("Type your message again to retry.");
                        pane.is_waiting = false;
                        pane.active_task = None;
                    }

                    StreamChunk::Info(msg) => {
                        log_info!("Info: {}", msg);
                    }
                }
            }
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

        if let Some(pane) = self.current_pane_mut() {
            pane.scroll_to_bottom();
        }
    }

    /// # scroll_to_bottom
    ///
    /// **Purpose:**
    /// Sets current pane's scroll position to maximum to show the most recent messages.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// None (mutates scroll state)
    pub fn scroll_to_bottom(&mut self) {
        if let Some(pane) = self.current_pane_mut() {
            pane.scroll = u16::MAX;
        }
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
            
            // Agent panel control
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
                let shutdown = self.enter_key();
                if shutdown {
                    return false;
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
                if let Some(pane) = self.current_pane_mut() {
                    pane.scroll = pane.scroll.saturating_sub(1);
                    pane.auto_scroll = false;  // User is manually scrolling
                }
                true
            }
            KeyCode::Down => {
                if let Some(pane) = self.current_pane_mut() {
                    pane.scroll = pane.scroll.saturating_add(1);
                }
                true
            }
            KeyCode::PageUp => {
                if let Some(pane) = self.current_pane_mut() {
                    pane.scroll = pane.scroll.saturating_sub(GLOBAL_CONFIG.tui.page_scroll_step);
                    pane.auto_scroll = false;
                }
                true
            }
            KeyCode::PageDown => {
                if let Some(pane) = self.current_pane_mut() {
                    pane.scroll = pane.scroll.saturating_add(GLOBAL_CONFIG.tui.page_scroll_step);
                }
                true
            }
            KeyCode::Esc => {
                return false;
            }
            _ => true,
        }
    }
    
    /// # enter_key
    ///
    /// **Purpose:**
    /// Processes the Enter key event, handling input commands and sending messages to agents.
    ///
    /// **Parameters:**
    /// None (uses self.input)
    ///
    /// **Returns:**
    /// `bool` - true if shutdown signal sent (app should exit), false otherwise
    ///
    /// **Details:**
    /// - Parses input through UserInput handler
    /// - Routes commands to appropriate handlers
    /// - Spawns async tasks for Grok API communication
    /// - Clears input field after processing
    fn enter_key(&mut self) -> bool {
        if self.input.trim().is_empty() {
            return false;
        }

        let line = self.input.trim().to_string();
        self.input.clear();

        let Some(user_input) = self.user_input.clone() else {
            self.add_message("No user input handler available.");
            return false;
        };

        match user_input.process_input(&line) {
            // Special cases that don't use the Command Pattern
            InputAction::DoNothing => {},
            InputAction::ContinueNoSend(msg) => {
                self.add_message(msg);
            }
            
            // All other actions use the Command Pattern
            action => {
                // Convert the InputAction into a Command object
                let command = from_input_action(action);
                
                // Execute the command and get the result
                let result = command.execute(self);
                
                // Handle the command result
                match result {
                    CommandResult::Continue => {},     // Keep running
                    CommandResult::Shutdown => return true,  // Exit application
                    CommandResult::Error(msg) => {
                        self.add_message(format!("Error: {}", msg));
                    }
                }
            }
        }

        false
    }
    
    /// # calculate_input_height
    ///
    /// **Purpose:**
    /// Calculates the required height for the input widget based on content and state.
    ///
    /// **Parameters:**
    /// - `width`: Available width for the input area
    ///
    /// **Returns:**
    /// `u16` - Height in terminal rows needed for the input widget
    ///
    /// **Details:**
    /// Returns 3 rows if waiting for response, otherwise calculates based on wrapped text
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
    
    /// # unified_messages
    ///
    /// **Purpose:**
    /// Converts unified message queue into formatted Lines for rendering.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// `Vec<Line>` - Vector of styled lines ready for ratatui rendering
    ///
    /// **Details:**
    /// User messages (starting with '>') are styled in light yellow and bold
    // Need to take out all the basic code that can be turned into functions for easier reading.
    fn unified_messages(&self) -> Vec<Line<'_>> {
        let mut lines: Vec<Line> = Vec::new();
        for unified in &self.unified_messages {
            let content = if unified.text.starts_with('>') {
                Line::from(Span::styled(
                    unified.text.clone(),
                    Style::default().fg(GLOBAL_CONFIG.tui.user_message_color).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(unified.text.clone())
            };
            lines.push(content);
        }
        lines
    }
    
    /// # pan_messages
    ///
    /// **Purpose:**
    /// Converts current pane's message queue into formatted Lines for rendering.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// `Vec<Line>` - Vector of styled lines for the current agent's messages
    ///
    /// **Details:**
    /// User messages (starting with '>') are styled in light yellow and bold
    fn pan_messages(&self) -> Vec<Line<'_>> {
        let mut lines: Vec<Line> = Vec::new();
        if let Some(pane) = self.current_pane() {
            for msg in &pane.messages {
                for line_text in msg.split('\n') {
                    let content = if msg.starts_with('>') {
                        Line::from(Span::styled(
                            line_text,
                            Style::default().fg(GLOBAL_CONFIG.tui.user_message_color).add_modifier(Modifier::BOLD),
                        ))
                    } else {
                        Line::from(line_text)
                    };
                    lines.push(content);
                }
            }
        }
        lines
    }
    
    /// # render_input
    ///
    /// **Purpose:**
    /// Renders the input widget with appropriate styling and content based on state.
    ///
    /// **Parameters:**
    /// - `frame`: The ratatui frame to render into
    /// - `area`: The rectangular area to render the input widget
    ///
    /// **Returns:**
    /// None (renders directly to frame)
    ///
    /// **Details:**
    /// Shows "Shadow is thinking..." animation when waiting, otherwise shows wrapped input text
    fn render_input(&self, frame: &mut Frame<'_>, area: Rect) {
        let is_waiting = self.current_pane()
            .map(|p| p.is_waiting)
            .unwrap_or(false);

        let dots = match self.current_pane()
            .map(|p| p.thinking_animation_frame)
            .unwrap_or(0) 
            {
                0 => "   ",
                1 => ".  ",
                2 => ".. ",
                3 => "...",
                _ => "   ",
            };

        let input_text = if is_waiting {
            Text::from(vec![
                Line::from(vec![
                    Span::styled(" > ", Style::default().fg(GLOBAL_CONFIG.tui.border_color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("Shadow is thinking...{}", dots), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
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
                            Span::styled(" > ", Style::default().fg(GLOBAL_CONFIG.tui.user_message_color)),
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
                    .border_style(Style::default().fg(GLOBAL_CONFIG.tui.border_color))
                    .title(" Input "),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(input_widget, area);
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

        // Setup input area
        let input_area = chunks[1];
    
        self.render_input(frame, input_area);
    

        // Gather messages from each pane and unified messages
        let pane_lines = self.pan_messages();
        let unified_lines = self.unified_messages();
        let mut global_scroll = self.scroll;
        let mut agent_scroll = self.current_pane()
                .map(|p| if p.auto_scroll { u16::MAX } else { p.scroll })
                .unwrap_or(0);

        render_message_section(
            frame,
            split[1],
            unified_lines,
            &capitalize_first("System"),
            &mut global_scroll,
        );

        let agent_name = self.get_agent_name(
            self.current_agent
                .unwrap_or(Uuid::nil())
        );
        let is_at_bottom = render_message_section(
            frame,
            split[0],
            pane_lines,
            &capitalize_first(&agent_name),
            &mut agent_scroll,
        );

        if let Some(pane) = self.current_pane_mut() {
            pane.scroll = agent_scroll;
            
           pane.auto_scroll = is_at_bottom;
        }

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

}