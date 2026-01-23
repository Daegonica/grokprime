//! # Daegonica Module: llm::client
//!
//! **Purpose:** Generic LLM connection coordinator
//!
//! **Context:**
//! - Works with ANY client that implements LlmClient trait
//! - Replaces specific GrokConnection/ClaudeConnection
//!
//! **Responsibilities:**
//! - Coordinate between client, conversation, and history
//! - Provide unified API for all LLM backends
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-21

use crate::prelude::*;
use crate::llm::LlmClient;
use std::path::Path;

/// Generic LLM connection that works with ANY client
#[derive(Debug, Clone)]
pub struct Connection<T: LlmClient> {
    client: T,
    pub conversation: GrokConversation,
    output: Option<SharedOutput>,
}

impl<T: LlmClient> Connection<T> {
    /// # new_without_output
    ///
    /// **Purpose:**
    /// Creates a new GrokConnection for TUI mode (no output handler).
    ///
    /// **Parameters:**
    /// - `persona`: The AI persona configuration
    ///
    /// **Returns:**
    /// Initialized GrokConnection ready for streaming
    ///
    /// **Errors / Failures:**
    /// - Panics if GROK_KEY environment variable not set
    /// - Logs warning if history load fails
    ///
    /// **Examples:**
    /// ```rust
    /// let persona = Arc::new(persona_config);
    /// let connection = GrokConnection::new_without_output(persona);
    /// ```
    pub fn new_without_output(client: T, persona: Arc<Persona>) -> Self {

        let conversation = if persona.enable_history {
            if let Ok(loaded_history) = HistoryManager::load_persona_history(&persona.name) {
                log_info!("Loaded history for {}: {} total messages",
                    persona.name, loaded_history.total_message_count);

                let messages = HistoryManager::build_history_from_loaded(&persona, loaded_history);
                GrokConversation::with_history(Arc::clone(&persona), messages)
            } else {
                log_info!("No history found for {}, starting fresh", persona.name);
                GrokConversation::new(persona)
            }
        } else {
            log_info!("History not enabled for {}", persona.name);
            GrokConversation::new(persona)
        };

        Connection {
            client,
            conversation,
            output: None,
        }
    }

    /// # new
    ///
    /// **Purpose:**
    /// Creates a new GrokConnection for CLI mode with output handler.
    ///
    /// **Parameters:**
    /// - `output`: Shared output handler for displaying messages
    /// - `persona`: The AI persona configuration
    ///
    /// **Returns:**
    /// Initialized GrokConnection for CLI usage
    ///
    /// **Examples:**
    /// ```rust
    /// let shadow = GrokConnection::new(Arc::new(CliOutput), persona);
    /// ```
    pub fn new(client: T,output: SharedOutput, persona: Arc<Persona>) -> Self {
        let mut conn = Self::new_without_output(client, persona);
        conn.output = Some(output);
        conn
    }

    /// # add_user_message
    ///
    /// **Purpose:**
    /// Adds a user message to the conversation.
    ///
    /// **Parameters:**
    /// - `content`: The user's message text
    ///
    /// **Returns:**
    /// None (delegates to conversation)
    pub fn add_user_message(&mut self, content: &str) {
        self.conversation.add_user_message(content);
    }

    /// # save_history
    ///
    /// **Purpose:**
    /// Saves conversation history to file.
    ///
    /// **Parameters:**
    /// - `path`: File path for raw history export
    ///
    /// **Returns:**
    /// `Result<(), std::io::Error>` - Success or I/O error
    ///
    /// **Examples:**
    /// ```rust
    /// connection.save_history("conversation_history.json")?;
    /// ```
    pub fn save_history(&self, path: &str) -> Result<(), std::io::Error> {
        HistoryManager::save_raw_history(&self.conversation.local_history, path)?;

        if let Some(ref output) = self.output {
            output.display(format!("Saved history ({} messages)", self.conversation.message_count()));
        } else {
            log_info!("Saved history ({} messages)", self.conversation.message_count());
        }

        Ok(())
    }

    /// # save_persona_history
    ///
    /// **Purpose:**
    /// Saves conversation to persona-specific history file.
    ///
    /// **Returns:**
    /// `Result<(), Box<dyn std::error::Error>>` - Success or error
    pub fn save_persona_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        HistoryManager::save_persona_history(&self.conversation)
    }

    /// # load_persona_history
    ///
    /// **Purpose:**
    /// Loads persona-specific history (static method for convenience).
    ///
    /// **Parameters:**
    /// - `persona_name`: Name of the persona
    ///
    /// **Returns:**
    /// `Result<ConversationHistory, Box<dyn std::error::Error>>` - Loaded history or error
    pub fn load_persona_history(persona_name: &str) -> Result<ConversationHistory, Box<dyn std::error::Error>> {
        HistoryManager::load_persona_history(persona_name)
    }

    /// # set_last_response_id
    ///
    /// **Purpose:**
    /// Updates the response ID (for backward compatibility).
    ///
    /// **Parameters:**
    /// - `id`: The response ID from API
    pub fn set_last_response_id(&mut self, id: String) {
        self.conversation.set_last_response_id(id);
    }

    /// # local_history (property access)
    ///
    /// **Purpose:**
    /// Provides access to conversation history for backward compatibility.
    ///
    /// **Returns:**
    /// Reference to local message history
    pub fn local_history(&self) -> &Vec<Message> {
        &self.conversation.local_history
    }

    /// # persona (property access)
    ///
    /// **Purpose:**
    /// Provides access to persona for backward compatibility.
    ///
    /// **Returns:**
    /// Reference to persona Arc
    pub fn persona(&self) -> &Arc<Persona> {
        &self.conversation.persona
    }

    /// # handle_response_streaming
    ///
    /// **Purpose:**
    /// Sends request and streams response chunks via channel (for TUI mode).
    ///
    /// **Parameters:**
    /// - `tx`: Channel sender for StreamChunk messages
    ///
    /// **Returns:**
    /// `Result<(), Box<dyn std::error::Error>>` - Success or error
    ///
    /// **Details:**
    /// - Builds request from conversation state
    /// - Sends via GrokClient
    /// - Updates conversation with response
    /// - Saves history if enabled
    /// - Triggers summarization if threshold reached
    pub async fn handle_response_streaming(
        &mut self,
        tx: mpsc::UnboundedSender<StreamChunk>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info!("Handling streaming response");

        let request = self.conversation.build_request();

        let response = self.client.send_streaming(&request, tx.clone()).await?;

        self.conversation.add_assistant_message(response.full_text);
        self.conversation.set_last_response_id(response.response_id.clone());

        if self.conversation.persona.enable_history {
            if let Err(e) = self.save_persona_history() {
                log_error!("Failed to save history: {}", e);
            }

            if self.conversation.should_summarize() {
                log_info!("History threshold reached, triggering summarization...");
                tx.send(StreamChunk::Info("Summarizing conversation history...".to_string()))?;

                if let Err(e) = self.summarize_history().await {
                    log_error!("Summarization failed: {}", e);
                    tx.send(StreamChunk::Error(format!("Summarization failed: {}", e)))?;
                } else {
                    if let Err(e) = self.save_persona_history() {
                        log_error!("Failed to save summarized history: {}", e);
                    }
                }
            }
        }

        tx.send(StreamChunk::Complete {
            response_id: response.response_id,
            full_reply: self.conversation.local_history.last()
                .map(|m| m.content.clone())
                .unwrap_or_default(),
        })?;

        Ok(())
    }

    /// # handle_response
    ///
    /// **Purpose:**
    /// Sends request and displays response synchronously (for CLI mode).
    ///
    /// **Returns:**
    /// `Result<(), Box<dyn std::error::Error>>` - Success or error
    pub async fn handle_response(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log_info!("Handling blocking response");

        let request = self.conversation.build_request();

        let print_stream = true;
        let response = self.client.send_blocking(&request, print_stream).await?;

        self.conversation.add_assistant_message(response.full_text);
        self.conversation.set_last_response_id(response.response_id);

        if self.conversation.persona.enable_history {
            if let Err(e) = self.save_persona_history() {
                log_error!("Failed to save history: {}", e);
            }
        }

        Ok(())
    }

    /// # summarize_history
    ///
    /// **Purpose:**
    /// Triggers conversation summarization using historian persona.
    ///
    /// **Returns:**
    /// `Result<(), Box<dyn std::error::Error>>` - Success or error
    ///
    /// **Details:**
    /// - Archives full history before summarization
    /// - Sends old messages to historian for summarization
    /// - Rebuilds history with summary + recent messages
    /// - Saves updated history
    pub async fn summarize_history(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let historian_path = "personas/historian/historian.yaml";
        let historian = match Persona::from_yaml_file(Path::new(historian_path)) {
            Ok(p) => Arc::new(p),
            Err(e) => {
                return Err(format!("Failed to load historian persona: {}", e).into());
            }
        };
        
        let limit = self.conversation.persona.history_message_limit;
        let cutoff_index = if self.conversation.local_history.len() > limit + 1 {
            self.conversation.local_history.len() - limit
        } else {
            return Ok(());
        };
        let messages_to_summarize = &self.conversation.local_history[1..cutoff_index];

        let formatted = messages_to_summarize
            .iter()
            .filter(|msg| !msg.content.contains("[Previous conversation summary:"))
            .map(|msg| format!("{}: {}", msg.role.to_uppercase(), msg.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let summary_prompt = format!(
            "Summarize this conversation:\n\n{}\n\nProvide a concise summary following your instructions.",
            formatted
        );

        log_info!("Sending {} messages to historian for summarization", messages_to_summarize.len());

        let summary_request = ChatRequest {
            model: "grok-4-fast".to_string(),
            input: vec![
                Message {
                    role: "system".to_string(),
                    content: historian.system_prompt.clone(),
                },
                Message {
                    role: "user".to_string(),
                    content: summary_prompt,
                },
            ],
            temperature: historian.temperature.unwrap_or(0.3),
            previous_response_id: None,
            stream: false,
        };

        let (tx, mut rx) = mpsc::unbounded_channel();
        let response = self.client.send_streaming(&summary_request, tx).await?;

        while rx.recv().await.is_some() {}

        let summary = response.full_text;
        log_info!("Summary generated: {}", summary);

        HistoryManager::archive_full_history(&self.conversation)?;

        let system_prompt = self.conversation.local_history[0].clone();
        let summary_message = Message {
            role: "system".to_string(),
            content: format!("[Previous conversation summary: {}]", summary),
        };

        let recent_messages = self.conversation.local_history[cutoff_index..].to_vec();

        let mut new_history = vec![system_prompt, summary_message];
        new_history.extend(recent_messages);

        log_info!("History rebuilt with summary. Messages: {} -> {}",
            self.conversation.local_history.len(), new_history.len());

        self.conversation.replace_history(new_history);

        Ok(())
    }

}
            