//! # Daegonica Module: grok::history
//!
//! **Purpose:** Conversation history persistence and file management
//!
//! **Context:**
//! - Handles all file I/O for conversation history
//! - Saves and loads persona-specific history files
//! - Manages history archiving for long conversations
//! - Does NOT manage in-memory state (that's in conversation module)
//!
//! **Responsibilities:**
//! - Load conversation history from JSON files
//! - Save conversation history to JSON files
//! - Archive complete conversation history
//! - Manage persona-specific history directories
//! - Handle ConversationHistory serialization/deserialization
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use crate::prelude::*;
use std::path::Path;

/// # HistoryManager
///
/// **Summary:**
/// Stateless utility for history file operations.
///
/// **Usage Example:**
/// ```rust
/// // Load history
/// let history = HistoryManager::load_persona_history("shadow")?;
///
/// // Save history
/// HistoryManager::save_persona_history(&conversation)?;
/// ```
pub struct HistoryManager;

impl HistoryManager {
    /// # load_persona_history
    ///
    /// **Purpose:**
    /// Loads saved conversation history for a specific persona.
    ///
    /// **Parameters:**
    /// - `persona_name`: Name of the persona (e.g., "shadow")
    ///
    /// **Returns:**
    /// `Result<ConversationHistory, Box<dyn std::error::Error>>` - Loaded history or error
    ///
    /// **File Location:**
    /// `personas/{persona_name}/history/{persona_name}_history.json`
    ///
    /// **Errors / Failures:**
    /// - File not found (no previous history)
    /// - Invalid JSON format
    /// - I/O errors reading file
    ///
    /// **Examples:**
    /// ```rust
    /// match HistoryManager::load_persona_history("shadow") {
    ///     Ok(history) => println!("Loaded {} messages", history.total_message_count),
    ///     Err(_) => println!("No history found, starting fresh"),
    /// }
    /// ```
    pub fn load_persona_history(persona_name: &str) -> Result<ConversationHistory, Box<dyn std::error::Error>> {
        let path = format!("personas/{}/history/{}_history.json", persona_name, persona_name);

        log_info!("Loading history from: {}", path);

        let content = std::fs::read_to_string(&path)?;
        let history: ConversationHistory = serde_json::from_str(&content)?;

        log_info!("Loaded history: {} total messages, {} recent messages",
            history.total_message_count, history.recent_messages.len());

        Ok(history)
    }

    /// # build_history_from_loaded
    ///
    /// **Purpose:**
    /// Converts a loaded ConversationHistory into a message Vec for GrokConversation.
    ///
    /// **Parameters:**
    /// - `persona`: The persona configuration (for system prompt)
    /// - `loaded_history`: The loaded ConversationHistory from file
    ///
    /// **Returns:**
    /// `Vec<Message>` - Complete message history ready for conversation
    ///
    /// **Details:**
    /// Builds: [system_prompt, optional_summary, recent_messages]
    ///
    /// **Examples:**
    /// ```rust
    /// let loaded = HistoryManager::load_persona_history("shadow")?;
    /// let messages = HistoryManager::build_history_from_loaded(&persona, loaded);
    /// let conversation = GrokConversation::with_history(persona, messages);
    /// ```
    pub fn build_history_from_loaded(persona: &Persona, loaded_history: ConversationHistory) -> Vec<Message> {
        let mut messages = vec![Message {
            role: "system".to_string(),
            content: persona.system_prompt.clone(),
        }];

        if let Some(summary) = loaded_history.summary {
            messages.push(Message {
                role: "system".to_string(),
                content: format!("[Previous conversation summary: {}]", summary),
            });
        }

        messages.extend(loaded_history.recent_messages);

        log_info!("Built history with {} total messages", messages.len());
        messages
    }

    /// # save_persona_history
    ///
    /// **Purpose:**
    /// Saves conversation history to persona-specific JSON file.
    ///
    /// **Parameters:**
    /// - `conversation`: The conversation to save
    ///
    /// **Returns:**
    /// `Result<(), Box<dyn std::error::Error>>` - Success or I/O error
    ///
    /// **File Location:**
    /// `personas/{persona_name}/history/{persona_name}_history.json`
    ///
    /// **Details:**
    /// - Creates directory if it doesn't exist
    /// - Saves only recent messages (based on persona.history_message_limit)
    /// - Preserves existing summary if present
    /// - Updates timestamp
    ///
    /// **Errors / Failures:**
    /// - Directory creation failures
    /// - File write permission errors
    /// - JSON serialization errors
    ///
    /// **Examples:**
    /// ```rust
    /// HistoryManager::save_persona_history(&conversation)?;
    /// ```
    pub fn save_persona_history(conversation: &GrokConversation) -> Result<(), Box<dyn std::error::Error>> {
        let persona_name = &conversation.persona.name;

        let dir_path = format!("personas/{}/history", persona_name);
        std::fs::create_dir_all(&dir_path)?;

        let limit = conversation.persona.history_message_limit;

        let recent_start = if conversation.local_history.len() > limit + 1 {
            conversation.local_history.len() - limit
        } else {
            1
        };

        let recent_messages: Vec<Message> = conversation.local_history[recent_start..].to_vec();

        let existing_summary = conversation.local_history.iter()
            .find(|msg| msg.role == "system" && msg.content.contains("[Previous conversation summary:"))
            .and_then(|msg| {
                msg.content
                    .strip_prefix("[Previous conversation summary: ")
                    .and_then(|s: &str| s.strip_suffix("]"))
                    .map(|s: &str| s.to_string())
            });

        let history = ConversationHistory {
            persona_name: persona_name.clone(),
            summary: existing_summary,
            recent_messages,
            total_message_count: conversation.local_history.len() -1,
            last_updated: chrono::Utc::now().to_rfc3339(),
            summarization_count: 0,
        };

        let json = serde_json::to_string_pretty(&history)?;
        let path = format!("personas/{}/history/{}_history.json", persona_name, persona_name);
        std::fs::write(&path, json)?;

        log_info!("Saved history for {} ({} messages)", persona_name, history.recent_messages.len());
        Ok(())
    }

    /// # save_raw_history
    ///
    /// **Purpose:**
    /// Saves complete raw message history to a file (for debugging or exports).
    ///
    /// **Parameters:**
    /// - `messages`: The message history to save
    /// - `path`: File path to write to
    ///
    /// **Returns:**
    /// `Result<(), std::io::Error>` - Success or I/O error
    ///
    /// **Examples:**
    /// ```rust
    /// HistoryManager::save_raw_history(&conversation.local_history, "backup.json")?;
    /// ```
    pub fn save_raw_history(messages: &[Message], path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(messages)?;
        std::fs::write(path, json)?;
        log_info!("Saved raw history to {} ({} messages)", path, messages.len());
        Ok(())
    }

    /// # archive_full_history
    ///
    /// **Purpose:**
    /// Archives complete conversation history with timestamp (before summarization).
    ///
    /// **Parameters:**
    /// - `conversation`: The conversation to archive
    ///
    /// **Returns:**
    /// `Result<(), Box<dyn std::error::Error>>` - Success or I/O error
    ///
    /// **File Location:**
    /// `personas/archives/{persona_name}_{timestamp}.json`
    ///
    /// **Details:**
    /// Creates timestamped archive before history is summarized/truncated
    ///
    /// **Examples:**
    /// ```rust
    /// // Before summarizing
    /// HistoryManager::archive_full_history(&conversation)?;
    /// ```
    pub fn archive_full_history(conversation: &GrokConversation) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all("personas/archives")?;

        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let path = format!("personas/archives/{}_{}.json", conversation.persona.name, timestamp);

        let json = serde_json::to_string_pretty(&conversation.local_history)?;
        std::fs::write(&path, json)?;

        log_info!("Archived full history for {} to {}", conversation.persona.name, path);
        Ok(())
    }

    /// # history_exists
    ///
    /// **Purpose:**
    /// Checks if a history file exists for a persona.
    ///
    /// **Parameters:**
    /// - `persona_name`: Name of the persona to check
    ///
    /// **Returns:**
    /// `bool` - true if history file exists, false otherwise
    ///
    /// **Examples:**
    /// ```rust
    /// if HistoryManager::history_exists("shadow") {
    ///     let history = HistoryManager::load_persona_history("shadow")?;
    /// }
    /// ```
    pub fn history_exists(persona_name: &str) -> bool {
        let path = format!("personas/{}/history/{}_history.json", persona_name, persona_name);
        Path::new(&path).exists()
    }

    /// # delete_history
    ///
    /// **Purpose:**
    /// Deletes the saved history file for a persona.
    ///
    /// **Parameters:**
    /// - `persona_name`: Name of the persona
    ///
    /// **Returns:**
    /// `Result<(), std::io::Error>` - Success or error if file doesn't exist
    ///
    /// **Examples:**
    /// ```rust
    /// HistoryManager::delete_history("shadow")?;
    /// ```
    pub fn delete_history(persona_name: &str) -> Result<(), std::io::Error> {
        let path = format!("personas/{}/history/{}_history.json", persona_name, persona_name);
        std::fs::remove_file(&path)?;
        log_info!("Deleted history for {}", persona_name);
        Ok(())
    }

}