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
use crate::persona::agent_manager::AgentManager;
use crate::persona::operations::AgentOperations;

pub trait AgentContext {
    fn get_agent_manager(&self) -> &AgentManager;
    fn get_agent_manager_mut(&mut self) -> &mut AgentManager;
    fn add_ui_message(&mut self, msg: String);
}

impl AgentContext for ShadowApp {
    fn get_agent_manager(&self) -> &AgentManager {
        &self.agent_manager
    }

    fn get_agent_manager_mut(&mut self) -> &mut AgentManager {
        &mut self.agent_manager
    }

    fn add_ui_message(&mut self, msg: String) {
        self.add_message(msg);
    }
}

impl AgentContext for AgentManager {
    fn get_agent_manager(&self) -> &AgentManager {
        self
    }

    fn get_agent_manager_mut(&mut self) -> &mut AgentManager {
        self
    }

    fn add_ui_message(&mut self, msg: String) {
        println!("{}", msg);
    }
}

pub trait AgentCommand: Debug {
    fn execute(&self, ctx: &mut dyn AgentContext) -> CommandResult;
}

pub trait TuiCommand: Debug {
    fn execute(&self, app: &mut ShadowApp) -> CommandResult;
}

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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult;
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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        let Some(agent) = ops.current_agent_info_mut() else {
            ops.display_message("No agent available. Create one with 'new <persona>'".to_string());
            return CommandResult::Continue;
        };

        agent.add_message(format!("> {}", self.content));
        agent.is_waiting = true;

        if let Some(old_task) = agent.active_task.take() {
            old_task.abort();
        }

        let mut connection = agent.connection.clone();
        let tx = agent.chunk_sender.clone();
        let content_owned = self.content.clone();

        let handle = tokio::spawn(async move {
            connection.add_user_message(&content_owned);
            if let Err(e) = connection.handle_response_streaming(tx.clone()).await {
                let _ = tx.send(StreamChunk::Error(format!("{}", e)));
            }
        });

        agent.active_task = Some(handle);
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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        let Some(agent) = ops.current_agent_info_mut() else {
            ops.display_message("No agent available to save history for.".to_string());
            return CommandResult::Continue;
        };

        let result = agent.connection.save_persona_history();
        let persona_name = agent.connection.conversation.persona.name.clone();

        match result {
            Ok(_) => {
                ops.display_message(format!("History saved for {}", persona_name));
                log_info!("History saved for {}", persona_name);
            }
            Err(e) => {
                log_error!("Failed to save history: {}", e);
                ops.display_message(format!("Failed to save history: {}", e));
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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        let Some(agent) = ops.current_agent_info_mut() else {
            ops.display_message("No agent available.".to_string());
            return CommandResult::Continue;
        };

        let msg_count = agent.connection.conversation.local_history.len();
        let has_summary = agent.connection.conversation.local_history.iter()
            .any(|m| m.content.contains("[Previous conversation summary:"));
        let persona_name = agent.connection.conversation.persona.name.clone();

        log_info!("{}: {} messages, Summary present: {}", persona_name, msg_count, has_summary);
        ops.display_message(format!(
            "History for {}: {} messages, Summary present: {}",
            persona_name, msg_count, has_summary
        ).to_string());

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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        let Some(agent) = ops.current_agent_info_mut() else {
            ops.display_message("No agent available.".to_string());
            return CommandResult::Continue;
        };

        let persona_name = agent.connection.conversation.persona.name.clone();
        let path = format!("history/{}_history.json", &persona_name);
        let result = std::fs::remove_file(&path);

        match result {
            Ok(_) => {
                log_info!("Cleared history for {}", persona_name);
                ops.display_message(format!("Cleared history for {}", persona_name));
            }
            Err(_) => {
                log_error!("No history for {}", persona_name);
                ops.display_message(format!("No history for {}", persona_name));
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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        if let Some(persona_ref) = ops.get_persona(&self.persona_name) {
            let id = Uuid::new_v4();
            ops.add_new_agent(id, persona_ref);
            ops.set_current_agent_id(Some(id));
            ops.display_message(format!(
                "Created new agent with persona '{}'",
                capitalize_first(&self.persona_name)
            ));
        } else {
            ops.display_message(format!(
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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        if let Some(id) = ops.get_current_agent_id() {
            ops.remove_agent(id);
            ops.display_message("Closed current agent.".to_string());
        } else {
            ops.display_message("No agent to close.".to_string());
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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        let mut status = String::new();
        status.push_str(&format!("Current agent: {}\n", ops.current_agent_info()
            .map(|agent| capitalize_first(&agent.persona_name))
            .unwrap_or("<none>".to_string())));

        status.push_str(&format!(" - Current agent: {}\n", ops.current_agent_info_mut()
            .map(|agent| capitalize_first(&agent.persona_name))
            .unwrap_or("<none>".to_string())));

        status.push_str(" - All agents:\n");
        let current_id = ops.get_current_agent_id();
        for (agent_id, agent_name) in ops.get_all_agent_names() {
            let marker = if Some(agent_id) == current_id { " ->"} else { " " };
            status.push_str(&format!("{} {}\n", marker, capitalize_first(&agent_name)));
        }
        status.push_str(&format!(" - Total tabs: {}", ops.get_agent_order().len()));

        ops.display_message(format!("{}", status));

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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        let Some(agent) = ops.current_agent_info_mut() else {
            ops.display_message("No agent available.".to_string());
            return CommandResult::Continue;
        };

        let mut conn = agent.connection.clone();
        let tx = agent.chunk_sender.clone();
        ops.display_message("Summarization started...".to_string());

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

        ops.display_message("Summarization task spawned.".to_string());
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
    fn execute(&self, _ops: &mut dyn AgentOperations) -> CommandResult {
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
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        let personas = vec!["shadow", "friday"];
        ops.display_message(format!("Available personas: {}", personas.join(", ")));
        CommandResult::Continue
    }
}

#[derive(Debug)]
struct UnimplementedCommand {
    feature: String,
}

impl Command for UnimplementedCommand {
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        ops.display_message(format!("Feature not yet implemented: {}", self.feature));
        CommandResult::Continue
    }
}

#[derive(Debug, Clone)]
struct TweetCommand {
    text: String,
}

impl Command for TweetCommand {
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        let Some(agent) = ops.current_agent_info_mut() else {
            ops.display_message("No agent available. Create one with 'new <persona>'".to_string());
            return CommandResult::Continue;
        };

        agent.add_message(r#"
            If I can't get the ai to respond to me quick enough with something big enough to actually trigger the autoscroll. Then I'll do it myself.
            I believe I've learned enough to actually code some useful test commands. I mean sure it's not the way most people would do it. But yet
            again I'm not most people and I didn't go to school for this shit. I'm just making up as I go and hoping the code works. So far so good
            I'd say! Alright this is probably enough I can always just spam this command.
            "#.to_string());
        CommandResult::Continue
    }
}

#[derive(Debug, Clone)]
struct DraftTweetCommand {
    text: String,
}

impl Command for DraftTweetCommand {
    fn execute(&self, ops: &mut dyn AgentOperations) -> CommandResult {
        let Some(agent) = ops.current_agent_info_mut() else {
            ops.display_message("No agent available. Create one with 'new <persona>'".to_string());
            return CommandResult::Continue;
        };

        let persona_name = agent.persona_name.clone();

        if persona_name == "viral" {
            agent.add_message(format!("> Tweet Draft: {}", self.text));
            agent.is_waiting = true;

            if let Some(old_task) = agent.active_task.take() {
                old_task.abort();
            }

            let mut connection = agent.connection.clone();
            let tx = agent.chunk_sender.clone();
            let text_owned = self.text.clone();

            let handle = tokio::spawn(async move {
                let define_tweet = format!(r#"
                    Please draft a tweet with the following content: "{}"
                    Keep it under 280 characters and suitable for Twitter.
                    Respond only with the tweet text, no additional commentary.
                    Use a casual and engaging tone.
                    Have at least one hashtag relevant to the content.
                    Have at least one mention of a relevant Twitter handle.
                    Prefer threads if necessary to fit the content.
                    Make it engaging and likely to get interactions.
                    Tag it with -Shadow at the end.
                    "#, text_owned);
                connection.add_user_message(&define_tweet);
                if let Err(e) = connection.handle_response_streaming(tx.clone()).await {
                    let _ = tx.send(StreamChunk::Error(format!("{}", e)));
                }
            });

            agent.active_task = Some(handle);
        } else {
            agent.add_message("Wrong Agent! Switch to Viral!")
        }

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
        InputAction::Quit                   => Box::new(QuitCommand::new()),
        InputAction::SendAsMessage(content) => Box::new(SendMessageCommand::new(content)),
        InputAction::SaveHistory            => Box::new(SaveHistoryCommand::new()),
        InputAction::HistoryInfo            => Box::new(HistoryInfoCommand::new()),
        InputAction::ClearHistory           => Box::new(ClearHistoryCommand::new()),
        InputAction::Summarize              => Box::new(SummarizeCommand::new()),
        InputAction::NewAgent(persona)      => Box::new(NewAgentCommand::new(persona)),
        InputAction::CloseAgent             => Box::new(CloseAgentCommand::new()),
        InputAction::AgentStatus            => Box::new(AgentStatusCommand::new()),
        InputAction::ListAgents             => Box::new(ListAgentsCommand::new()),
        InputAction::PostTweet(text)        => Box::new(TweetCommand {text}),
        InputAction::DraftTweet(text)       => Box::new(DraftTweetCommand {text}),
        InputAction::DoNothing | InputAction::ContinueNoSend(_) => {
            Box::new(UnimplementedCommand {
                feature: "Hey dumbass, these do nothing".to_string(),
            })
        }
    }
}