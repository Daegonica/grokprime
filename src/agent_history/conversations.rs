//! # Daegonica Module: grok::conversation
//!
//! **Purpose:** In-memory conversation state and message management
//!
//! **Context:**
//! - Maintains the conversation history in memory
//! - Builds ChatRequest payloads for the API
//! - Tracks response IDs for conversation continuity
//! - Does NOT handle file I/O (that's in history module)
//!
//! **Responsibilities:**
//! - Store messages in local_history Vec
//! - Add user and assistant messages
//! - Build API request payloads with correct context
//! - Manage response ID for conversation threading
//! - Determine when summarization is needed
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use crate::prelude::*;

/// # GrokConversation
///
/// **Summary:**
/// In-memory conversation state manager.
///
/// **Fields:**
/// - `local_history`: Complete message history (system prompt + all messages)
/// - `last_response_id`: Grok's last response ID for threading
/// - `persona`: The AI persona configuration for this conversation
///
/// **Usage Example:**
/// ```rust
/// let persona = Arc::new(persona_config);
/// let mut conversation = GrokConversation::new(persona);
/// conversation.add_user_message("Hello!");
/// let request = conversation.build_request();
/// ```
#[derive(Debug, Clone)]
pub struct GrokConversation {
    pub local_history: Vec<Message>,
    last_response_id: Option<String>,
    pub persona: Arc<Persona>,
}

impl GrokConversation {
    /// # new
    ///
    /// **Purpose:**
    /// Creates a new conversation with system prompt from persona.
    ///
    /// **Parameters:**
    /// - `persona`: The AI persona configuration
    ///
    /// **Returns:**
    /// Initialized conversation with system prompt in history
    ///
    /// **Examples:**
    /// ```rust
    /// let persona = Arc::new(persona_config);
    /// let conversation = GrokConversation::new(persona);
    /// ```
    pub fn new(persona: Arc<Persona>) -> Self {
        let sys_message = Message {
            role: "system".to_string(),
            content: persona.system_prompt.clone(),
        };

        let local_history = vec![sys_message];

        GrokConversation {
            local_history,
            last_response_id: None,
            persona,
        }
    }

    /// # with_history
    ///
    /// **Purpose:**
    /// Creates a conversation initialized with loaded history.
    ///
    /// **Parameters:**
    /// - `persona`: The AI persona configuration
    /// - `history`: Previously loaded conversation history
    ///
    /// **Returns:**
    /// Conversation with full history restored
    ///
    /// **Examples:**
    /// ```rust
    /// let loaded = HistoryManager::load_persona_history("shadow")?;
    /// let conversation = GrokConversation::with_history(persona, loaded);
    /// ```
    pub fn with_history(persona: Arc<Persona>, loaded_history: Vec<Message>) -> Self {
        GrokConversation {
            local_history: loaded_history,
            last_response_id: None,
            persona,
        }
    }

    /// # add_user_message
    ///
    /// **Purpose:**
    /// Adds a user message to the conversation history.
    ///
    /// **Parameters:**
    /// - `content`: The user's message text
    ///
    /// **Returns:**
    /// None (mutates local_history)
    ///
    /// **Examples:**
    /// ```rust
    /// conversation.add_user_message("What is Rust?");
    /// ```
    pub fn add_user_message(&mut self, content: &str) {

        let new_msg = Message {
            role: "user".to_string(),
            content: content.to_string(),
        };

        self.local_history.push(new_msg);
    }
    
    /// # add_assistant_message
    ///
    /// **Purpose:**
    /// Adds an assistant response to the conversation history.
    ///
    /// **Parameters:**
    /// - `content`: The assistant's response text
    ///
    /// **Returns:**
    /// None (mutates local_history)
    ///
    /// **Examples:**
    /// ```rust
    /// conversation.add_assistant_message(response.full_text);
    /// ```
    pub fn add_assistant_message(&mut self, content: String) {

        let msg = Message {
            role: "assistant".to_string(),
            content,
        };

        self.local_history.push(msg);
    }

    /// # set_last_response_id
    ///
    /// **Purpose:**
    /// Updates the response ID for conversation continuity.
    ///
    /// **Parameters:**
    /// - `id`: The response ID from Grok API
    ///
    /// **Returns:**
    /// None (mutates last_response_id)
    pub fn set_last_response_id(&mut self, id: String) {
        self.last_response_id = Some(id);
    }

    /// # get_last_response_id
    ///
    /// **Purpose:**
    /// Retrieves the current response ID if one exists.
    ///
    /// **Returns:**
    /// `Option<&String>` - Reference to response ID or None
    pub fn get_last_response_id(&self) -> Option<&String> {
        self.last_response_id.as_ref()
    }

    /// # build_request
    ///
    /// **Purpose:**
    /// Builds a ChatRequest payload for the API based on conversation state.
    ///
    /// **Details:**
    /// - If no response_id: Sends full history (new conversation or first message)
    /// - If response_id exists: Only sends the last user message (conversation threading)
    ///
    /// **Returns:**
    /// ChatRequest ready to send to GrokClient
    ///
    /// **Examples:**
    /// ```rust
    /// let request = conversation.build_request();
    /// let response = client.send_streaming_request(&request, tx).await?;
    /// ```
    pub fn build_request(&self) -> ChatRequest {
        let input = if self.last_response_id.is_none() {
            log_info!("Building request with full history ({} messages)", self.local_history.len());
            self.local_history.clone()
        } else {
            if let Some(last_msg) = self.local_history.last() {
                log_info!("Building request with last message only (threaded conversation)");
                vec![last_msg.clone()]
            } else {
                log_error!("No messages in history despite response ID existing!");
                vec![]
            }
        };

        ChatRequest {
            model: GLOBAL_CONFIG.grok.model_name.to_string(),
            input,
            temperature: self.persona.temperature.unwrap_or(GLOBAL_CONFIG.grok.default_temperature),
            previous_response_id: self.last_response_id.clone(),
            stream: GLOBAL_CONFIG.grok.stream_enabled,
        }
    }

    /// # should_summarize
    ///
    /// **Purpose:**
    /// Determines if conversation has reached summarization threshold.
    ///
    /// **Details:**
    /// Counts only user/assistant messages (excludes system and summary messages)
    ///
    /// **Returns:**
    /// `bool` - true if summarization should be triggered
    ///
    /// **Examples:**
    /// ```rust
    /// if conversation.should_summarize() {
    ///     // Trigger summarization...
    /// }
    /// ```
    pub fn should_summarize(&self) -> bool {
        if !self.persona.enable_history {
            return false;
        }

        let message_count = self.local_history.iter()
            .filter(|msg| msg.role != "system" || !msg.content.contains("[Previous conversation summary:"))
            .count();

        let threshold_exceeded = message_count > self.persona.summary_threshold;

        if threshold_exceeded {
            log_info!("Summarization threshold reached: {} > {}",
                message_count, self.persona.summary_threshold);
        }

        threshold_exceeded
    }

    /// # message_count
    ///
    /// **Purpose:**
    /// Returns total number of messages in history.
    ///
    /// **Returns:**
    /// `usize` - Count of all messages (including system prompts)
    pub fn message_count(&self) -> usize {
        self.local_history.len()
    }

    /// # get_system_prompt
    ///
    /// **Purpose:**
    /// Retrieves the system prompt message (always first in history).
    ///
    /// **Returns:**
    /// `Option<&Message>` - Reference to system prompt or None if history empty
    pub fn get_system_prompt(&self) -> Option<&Message> {
        self.local_history.first()
    }

    /// # clear_history
    ///
    /// **Purpose:**
    /// Resets conversation to just the system prompt.
    ///
    /// **Returns:**
    /// None (mutates local_history)
    ///
    /// **Examples:**
    /// ```rust
    /// conversation.clear_history();  // Start fresh conversation
    /// ```
    pub fn clear_history(&mut self) {
        let system_prompt = self.get_system_prompt().cloned();

        if let Some(prompt) = system_prompt {
            self.local_history = vec![prompt];
            self.last_response_id = None;
            log_info!("Conversation history cleared");
        } else {
            log_error!("Cannot clear history - no system prompt found!");
        }
    }

    /// # replace_history
    ///
    /// **Purpose:**
    /// Replaces the entire conversation history (used after summarization).
    ///
    /// **Parameters:**
    /// - `new_history`: The new message history to use
    ///
    /// **Returns:**
    /// None (replaces local_history)
    ///
    /// **Examples:**
    /// ```rust
    /// // After summarization
    /// let new_history = vec![system_prompt, summary_msg, recent_msgs...];
    /// conversation.replace_history(new_history);
    /// ```
    pub fn replace_history(&mut self, new_history: Vec<Message>) {
        let old_len = self.local_history.len();
        self.local_history = new_history;
        log_info!("History replaced: {} messages -> {} messages", old_len, self.local_history.len());
    }

}