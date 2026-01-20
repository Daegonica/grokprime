//! # Daegonica Module: commands
//!
//! **Purpose:** Command pattern implementation for user actions
//!
//! **Context:**
//! - Implements the Gang of Four Command pattern
//! - Encapsulates user actions as first-class objects
//! - Enables undo/redo, command queuing, and logging
//!
//! **Responsibilities:**
//! - Define the Command trait interface
//! - Implement concrete command types for each action
//! - Provide command execution and result handling
//! - Enable polymorphic command treatment
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use crate::prelude::*;
use crate::tui::ShadowApp;
use std::fmt::Debug;
use uuid::Uuid;

/// # Command
///
/// **Summary:**
/// Trait defining the interface for all executable commands.
///
/// **Methods:**
/// - `execute`: Performs the command's action
///
/// **Usage Example:**
/// ```rust
/// let cmd = SendMessageCommand::new("Hello".to_string());
/// let result = cmd.execute(&mut app)?;
/// ```
///
/// **Design Pattern:**
/// This is the Command pattern from Gang of Four design patterns.
/// Commands encapsulate actions as objects, enabling:
/// - Polymorphic treatment of all commands
/// - Command queuing and scheduling
/// - Undo/redo functionality (future)
/// - Logging and auditing
pub trait Command: Debug {
    /// Executes the command and returns a result.
    ///
    /// # Parameters
    /// - `app`: Mutable reference to the application state
    ///
    /// # Returns
    /// - `CommandResult`: The outcome of the command execution
    fn execute(&self, app: &mut ShadowApp) -> CommandResult;
}

/// # CommandResult
///
/// **Summary:**
/// Enum representing the outcome of command execution.
///
/// **Variants:**
/// - `Continue`: Command succeeded, continue normal operation
/// - `Shutdown`: Command succeeded, application should exit
/// - `Error(String)`: Command failed with error message
///
/// **Usage Example:**
/// ```rust
/// match cmd.execute(&mut app) {
///     CommandResult::Continue => { /* keep running */ }
///     CommandResult::Shutdown => { /* exit app */ }
///     CommandResult::Error(msg) => { /* show error */ }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    Continue,
    Shutdown,
    Error(String)
}

/// # SendMessageCommand
///
/// **Summary:**
/// Command to send a message to the current agent via Grok API.
///
/// **Fields:**
/// - `content`: The message text to send
///
/// **Usage Example:**
/// ```rust
/// let cmd = SendMessageCommand::new("Hello Shadow!".to_string());
/// cmd.execute(&mut app)?;
/// ```
#[derive(Debug, Clone)]
pub struct SendMessageCommand {
    content: String,
}

impl SendMessageCommand {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl Command for SendMessageCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        let Some(pane) = app.current_pane_mut() else {
            app.add_message("No agent available. Create one with 'new <persona>'");
            return CommandResult::Continue;
        };

        pane.add_message(format!("> {}", self.content));
        pane.is_waiting = true;

        if let Some(old_task) = pane.active_task.take() {
            old_task.abort();
        }

        let mut connection = pane.connection.clone();
        let tx = pane.chunk_sender.clone();
        let content_owned = self.content.clone();

        let handle = tokio::spawn(async move {
            connection.add_user_message(&content_owned);
            if let Err(e) = connection.handle_response_streaming(tx.clone()).await {
                let _ = tx.send(StreamChunk::Error(format!("{}", e)));
            }
        });

        pane.active_task = Some(handle);
        CommandResult::Continue
    }
}

/// # SaveHistoryCommand
///
/// **Summary:**
/// Command to save the current agent's conversation history to disk.
#[derive(Debug, Clone)]
pub struct SaveHistoryCommand;

impl SaveHistoryCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for SaveHistoryCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        let Some(pane) = app.current_pane_mut() else {
            app.add_message("No agent available to save history for.");
            return CommandResult::Continue;
        };

        let result = pane.connection.save_persona_history();
        let persona_name = pane.connection.persona.name.clone();

        match result {
            Ok(_) => {
                app.add_message(format!("History saved for {}", persona_name));
                log_info!("History saved for {}", persona_name);
            }
            Err(e) => {
                log_error!("Failed to save history: {}", e);
                app.add_message(format!("Failed to save history: {}", e));
            }
        }

        CommandResult::Continue
    }
}

/// # HistoryInfoCommand
///
/// **Summary:**
/// Command to display information about the current agent's conversation history.
#[derive(Debug, Clone)]
pub struct HistoryInfoCommand;

impl HistoryInfoCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for HistoryInfoCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        let Some(pane) = app.current_pane_mut() else {
            app.add_message("No agent available.");
            return CommandResult::Continue;
        };

        let msg_count = pane.connection.local_history.len();
        let has_summary = pane.connection.local_history.iter()
            .any(|m| m.content.contains("[Previous conversation summary:"));
        let persona_name = pane.connection.persona.name.clone();

        log_info!("{}: {} messages, Summary present: {}", persona_name, msg_count, has_summary);
        app.add_message(format!(
            "History for {}: {} messages, Summary present: {}",
            persona_name, msg_count, has_summary
        ));

        CommandResult::Continue
    }
}

/// # ClearHistoryCommand
///
/// **Summary:**
/// Command to clear the history file for the current agent from disk.
#[derive(Debug, Clone)]
pub struct ClearHistoryCommand;

impl ClearHistoryCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ClearHistoryCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        let Some(pane) = app.current_pane_mut() else {
            app.add_message("No agent available.");
            return CommandResult::Continue;
        };

        let persona_name = pane.connection.persona.name.clone();
        let path = format!("history/{}_history.json", &persona_name);
        let result = std::fs::remove_file(&path);

        match result {
            Ok(_) => {
                log_info!("Cleared history for {}", persona_name);
                app.add_message(format!("Cleared history for {}", persona_name));
            }
            Err(_) => {
                log_error!("No history for {}", persona_name);
                app.add_message(format!("No history for {}", persona_name));
            }
        }

        CommandResult::Continue
    }
}

/// # NewAgentCommand
///
/// **Summary:**
/// Command to create a new agent with a specified persona.
///
/// **Fields:**
/// - `persona_name`: Name of the persona to load and instantiate
#[derive(Debug, Clone)]
pub struct NewAgentCommand {
    persona_name: String,
}

impl NewAgentCommand {
    pub fn new(persona_name: String) -> Self {
        Self {
            persona_name
        }
    }
}

impl Command for NewAgentCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        if let Some(persona_ref) = app.personas.get(&self.persona_name) {
            let id = Uuid::new_v4();
            app.add_agent(id, Arc::clone(persona_ref));
            app.current_agent = Some(id);
            app.add_message(format!(
                "Created new agent with persona '{}'",
                capitalize_first(&self.persona_name)
            ));
        } else {
            app.add_message(format!(
                "Persona '{}' not found.",
                capitalize_first(&self.persona_name)
            ));
        }

        CommandResult::Continue
    }
}

/// # CloseAgentCommand
///
/// **Summary:**
/// Command to close the current agent and remove it from the application.
#[derive(Debug, Clone)]
pub struct CloseAgentCommand;

impl CloseAgentCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for CloseAgentCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        if let Some(id) = app.current_agent {
            app.remove_agent(id);
            app.add_message("Closed current agent.");
        } else {
            app.add_message("No agent to close.");
        }

        CommandResult::Continue
    }
}

/// # AgentStatusCommand
///
/// **Summary:**
/// Command to display status information about all agents.
#[derive(Debug, Clone)]
pub struct AgentStatusCommand;

impl AgentStatusCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for AgentStatusCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        let mut status = String::new();
        status.push_str(&format!("Current agent: {}\n", app.current_agent
            .and_then(|id| app.agents.get(&id))
            .map(|pane| capitalize_first(&pane.persona_name))
            .unwrap_or("<none>".to_string())));

        status.push_str(&format!(" - Current pane: {}\n", app.current_pane_mut()
            .map(|pane| capitalize_first(&pane.persona_name))
            .unwrap_or("<none>".to_string())));

        status.push_str(" - All agents:\n");

        for id in &app.agent_order {
            let pane = &app.agents[id];
            let marker = if Some(*id) == app.current_agent {" ->"} else {" "};
            status.push_str(&format!("{} {}\n", marker, capitalize_first(&pane.persona_name)));
        }
        status.push_str(&format!(" - Total tabs: {}", app.agent_order.len()));

        app.add_message(format!("{}", status));

        CommandResult::Continue
    }
}

/// # SummarizeCommand
///
/// **Summary:**
/// Command to trigger conversation history summarization for the current agent.
#[derive(Debug, Clone)]
pub struct SummarizeCommand;

impl SummarizeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for SummarizeCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        let Some(pane) = app.current_pane_mut() else {
            app.add_message("No agent available.");
            return CommandResult::Continue;
        };

        let mut conn = pane.connection.clone();
        let tx = pane.chunk_sender.clone();
        app.add_message("Summarization started...");

        tokio::spawn(async move {
            tx.send(StreamChunk::Info("Starting summarization...".to_string())).ok();
            if let Err(e) = conn.summarize_history().await {
                tx.send(StreamChunk::Error(format!("Summarization error: {}", e))).ok();
            } else {
                tx.send(StreamChunk::Info("Summarization complete.".to_string())).ok();
                if let Err(e) = conn.save_persona_history() {
                    tx.send(StreamChunk::Error(format!("Failed to save persona history: {}", e))).ok();
                }
            }
        });

        app.add_message("Summarization task spawned.");
        CommandResult::Continue
    }
}

/// # QuitCommand
///
/// **Summary:**
/// Command to gracefully shut down the application.
#[derive(Debug, Clone)]
pub struct QuitCommand;

impl QuitCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for QuitCommand {
    fn execute(&self, _app: &mut ShadowApp) -> CommandResult {
        CommandResult::Shutdown
    }
}

#[derive(Debug, Clone)]
pub struct ListAgentsCommand;

impl ListAgentsCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ListAgentsCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        let personas = vec!["shadow", "friday"];
        app.add_message(format!("Available personas: {}", personas.join(", ")));
        CommandResult::Continue
    }
}

#[derive(Debug)]
struct UnimplementedCommand {
    feature: String,
}

impl Command for UnimplementedCommand {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult {
        app.add_message(format!("Feature not yet implemented: {}", self.feature));
        CommandResult::Continue
    }
}

/// # from_input_action
///
/// **Purpose:**
/// Converts an InputAction into a boxed Command trait object.
///
/// **Parameters:**
/// - `action`: The InputAction to convert
///
/// **Returns:**
/// - `Option<Box<dyn Command>>`: Boxed command if action is supported, None otherwise
///
/// **Usage Example:**
/// ```rust
/// if let Some(cmd) = from_input_action(action) {
///     let result = cmd.execute(&mut app);
/// }
/// ```
pub fn from_input_action(action: InputAction) -> Box<dyn Command> {
    match action {
        InputAction::Quit => Box::new(QuitCommand::new()),
        InputAction::SendAsMessage(content) => Box::new(SendMessageCommand::new(content)),
        InputAction::SaveHistory => Box::new(SaveHistoryCommand::new()),
        InputAction::HistoryInfo => Box::new(HistoryInfoCommand::new()),
        InputAction::ClearHistory => Box::new(ClearHistoryCommand::new()),
        InputAction::Summarize => Box::new(SummarizeCommand::new()),
        InputAction::NewAgent(persona) => Box::new(NewAgentCommand::new(persona)),
        InputAction::CloseAgent => Box::new(CloseAgentCommand::new()),
        InputAction::AgentStatus => Box::new(AgentStatusCommand::new()),
        InputAction::ListAgents => Box::new(ListAgentsCommand::new()),
        InputAction::PostTweet(text) => Box::new(UnimplementedCommand {
            feature: format!("Tweet: {}", text),
        }),
        InputAction::DraftTweet(text) => Box::new(UnimplementedCommand {
            feature: format!("Draft tweet: {}", text),
        }),
        InputAction::DoNothing | InputAction::ContinueNoSend(_) => {
            Box::new(UnimplementedCommand {
                feature: "Internal action".to_string(),
            })
        }
    }
}