//! # Daegonica Module: user::user_input
//!
//! **Purpose:** User command parsing and input action determination
//!
//! **Context:**
//! - Processes raw user input into actionable commands
//! - Distinguishes between chat messages and system commands
//! - Used by both TUI and CLI modes
//!
//! **Responsibilities:**
//! - Read user input from stdin
//! - Parse commands with arguments
//! - Convert input to InputAction enum variants
//! - Validate command syntax and provide usage hints
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use crate::prelude::*;
use strum::{EnumString, IntoStaticStr, EnumIter};
use std::str::FromStr;

/// # UserInput
///
/// **Summary:**
/// Handler for reading and processing user input with system context awareness.
///
/// **Fields:**
/// - `os_info`: System information for context-aware commands
/// - `output`: Shared output handler for displaying messages
///
/// **Usage Example:**
/// ```rust
/// let mut user_input = UserInput::new(Arc::clone(&output));
/// if let Some(input) = user_input.read_user_input()? {
///     let action = user_input.process_input(&input);
/// }
/// ```
#[derive(Clone)]
pub struct UserInput {
    os_info: OsInfo,
    output: SharedOutput,
}

impl std::fmt::Debug for UserInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserInput")
            .field("os_info", &self.os_info)
            .field("output", &"<OutputHandler>")
            .finish()
    }
}

impl UserInput {

    /// # new
    ///
    /// **Purpose:**
    /// Creates a new UserInput handler with current system information.
    ///
    /// **Parameters:**
    /// - `output`: Shared output handler for displaying messages
    ///
    /// **Returns:**
    /// Initialized UserInput ready to process commands
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    ///
    /// **Examples:**
    /// ```rust
    /// let user_input = UserInput::new(Arc::clone(&output));
    /// ```
    pub fn new(output: SharedOutput) -> Self {
        let os_info = OsInfo::new();
        UserInput{ os_info, output}
    }

    /// # read_user_input
    ///
    /// **Purpose:**
    /// Reads a line of input from the user via stdin.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// `Result<Option<String>, std::io::Error>` - Some(input) if non-empty, None if empty
    ///
    /// **Errors / Failures:**
    /// - Stdin read errors
    /// - Terminal input issues
    ///
    /// **Examples:**
    /// ```rust
    /// match user_input.read_user_input()? {
    ///     Some(input) => process(input),
    ///     None => continue,
    /// }
    /// ```
    pub fn read_user_input(&mut self) -> std::io::Result<Option<String>> {
        print!("Input: ");
        io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    }

    /// # process_input
    ///
    /// **Purpose:**
    /// Parses raw input string into an InputAction based on command recognition.
    ///
    /// **Parameters:**
    /// - `raw_input`: The raw string entered by the user
    ///
    /// **Returns:**
    /// InputAction representing the parsed command or message
    ///
    /// **Errors / Failures:**
    /// - None (unrecognized commands become SendAsMessage)
    ///
    /// **Examples:**
    /// ```rust
    /// let action = user_input.process_input("tweet Hello world!");
    /// match action {
    ///     InputAction::PostTweet(text) => post_tweet(text),
    ///     _ => {}
    /// }
    /// ```
    pub fn process_input(&self, raw_input: &str) -> InputAction {
        let parts: Vec<&str> = raw_input.splitn(2, ' ').collect();
        let potential_command = parts[0];
        let remainder = if parts.len() > 1 { parts[1] } else { "" };

        let cmd = UserCommand::from_str(potential_command).unwrap_or(UserCommand::Unknown);

        match cmd {
            // System OS info command
            UserCommand::System => {
                let output_text = self.os_info.display_all();
                InputAction::ContinueNoSend(output_text)
            },

            // Shutdown command
            UserCommand::Quit | UserCommand::Exit => InputAction::Quit,

            // Twitter related commands
            UserCommand::Tweet => {
                if remainder.is_empty() {
                    self.output.display("Usage: tweet <your message>".to_string());
                    InputAction::DoNothing
                } else {
                    InputAction::PostTweet(remainder.to_string())
                }
            },
            UserCommand::Draft => {
                if remainder.is_empty() {
                    self.output.display("Usage: draft <your idea>".to_string());
                    InputAction::DoNothing
                } else {
                    InputAction::DraftTweet(remainder.to_string())
                }
            },

            // Agent management commands
            UserCommand::Status => {
                InputAction::AgentStatus
            }
            UserCommand::New => {
                if remainder.is_empty() {
                    self.output.display("Usage: new <persona>".to_string());
                    InputAction::DoNothing
                } else {
                    InputAction::NewAgent(remainder.to_string())
                }
            },
            UserCommand::Close => InputAction::CloseAgent,
            UserCommand::List => InputAction::ListAgents,

            // Send as regular message to agent
            UserCommand::Unknown => InputAction::SendAsMessage(raw_input.to_string()),
        }
    }

}

/// # UserCommand
///
/// **Summary:**
/// Internal enum for recognized user commands with case-insensitive parsing.
///
/// **Variants:**
/// - `System`: Display system information
/// - `Quit`: Exit the application
/// - `Exit`: Alternative exit command
/// - `Tweet`: Post a tweet with given text
/// - `Draft`: Generate a tweet draft from an idea
/// - `New`: Create a new agent with specified persona
/// - `Close`: Close the current agent
/// - `List`: List all active agents
/// - `Unknown`: Unrecognized command (fallback)
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr, EnumIter)]
#[strum(serialize_all = "lowercase")]
#[strum(ascii_case_insensitive)]
enum UserCommand {
    System,
    Quit,
    Exit,
    Tweet,
    Draft,
    New,
    Close,
    List,
    Status,

    #[strum(disabled)]
    Unknown,
}
