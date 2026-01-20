//! # Daegonica Module: grok::agent
//!
//! **Purpose:** Core Grok API client and conversation management
//!
//! **Context:**
//! - Primary interface for interacting with the Grok AI service
//! - Maintains conversation history and API state
//! - Used by both TUI and CLI modes
//!
//! **Responsibilities:**
//! - Authenticate with Grok API using environment credentials
//! - Send chat requests and process responses
//! - Manage conversation history and persistence
//! - Handle API errors gracefully
//! - Maintain system prompt and personality configuration
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------
use futures_util::StreamExt;
use crate::prelude::*;

/// # GrokConnection
///
/// **Summary:**
/// Client connection to the Grok API with state management for ongoing conversations.
///
/// **Fields:**
/// - `api_key`: Authentication key for Grok API (from GROK_KEY env var)
/// - `client`: HTTP client for making API requests
/// - `request`: Current chat request with conversation context
/// - `last_response_id`: ID of the last response for conversation continuity
/// - `local_history`: Complete conversation history including system prompts
/// - `output`: Shared output handler for displaying messages
/// - `persona`: The AI persona configuration for this connection
///
/// **Usage Example:**
/// ```rust
/// let persona = Arc::new(persona_config);
/// let mut shadow = GrokConnection::new(Arc::clone(&output), persona);
/// shadow.add_user_message("Hello!");
/// shadow.handle_response().await?;
/// ```
#[derive(Debug, Clone)]
pub struct GrokConnection {
    api_key: String,
    client: Client,
    request: ChatRequest,
    last_response_id: Option<String>,
    pub local_history: Vec<Message>,
    output: Option<SharedOutput>,
    pub persona: Arc<Persona>,
}


impl GrokConnection {

    /// # new_without_output
    ///
    /// **Purpose:**
    /// Creates a new GrokConnection instance with loaded history or fresh system prompt.
    ///
    /// **Parameters:**
    /// - `persona`: The AI persona configuration (Arc-wrapped for sharing)
    ///
    /// **Returns:**
    /// Initialized GrokConnection ready for conversation
    ///
    /// **Errors / Failures:**
    /// - Panics if GROK_KEY environment variable is not set
    /// - Logs warning if history file is invalid or empty
    ///
    /// **Examples:**
    /// ```rust
    /// let persona = Arc::new(persona_config);
    /// let shadow = GrokConnection::new_without_output(persona);
    /// ```
    pub fn new_without_output(persona: Arc<Persona>) -> Self {
        dotenv().ok();
        let api_key = env::var("GROK_KEY").expect("GROK_KEY not set");

        // Make this a loadable personality set.
        let sys_messages = Message {
                role: "system".to_string(),
                content: persona.system_prompt.clone(),
            };

        let mut local_history = vec![sys_messages.clone()];
        if persona.enable_history {
            if let Ok(loaded_history) = Self::load_persona_history(&persona.name) {
                log_info!("Loaded History for {}: {} total messages",
                    persona.name, loaded_history.total_message_count);

                if let Some(summary) = loaded_history.summary {
                    local_history.push(Message {
                        role: "system".to_string(),
                        content: format!("[Previous conversation summary: {}]", summary),
                    });
                }

                local_history.extend(loaded_history.recent_messages);
            } else {
                log_info!("No history found for {} or load failed, starting fresh", persona.name);
            }
        }

        let request = ChatRequest {
            model: GLOBAL_CONFIG.grok.model_name.to_string(),
            input: Vec::new(),  // Start empty, populate in add_user_message
            temperature: persona.temperature.unwrap_or(GLOBAL_CONFIG.grok.default_temperature),
            previous_response_id: None,
            stream: GLOBAL_CONFIG.grok.stream_enabled,
        };

        GrokConnection{
            api_key, 
            client: Client::new(), 
            request,
            last_response_id: None,
            local_history,
            output: None,
            persona,
        }
    } 

    /// # new
    ///
    /// **Purpose:**
    /// Creates GrokConnection for CLI mode with output handler for blocking display.
    ///
    /// **Parameters:**
    /// - `output`: Shared output handler for displaying messages
    ///
    /// **Returns:**
    /// Initialized GrokConnection ready for CLI usage
    ///
    /// **Examples:**
    /// ```rust
    /// let shadow = GrokConnection::new(Arc::new(CliOutput));
    /// ```
    pub fn new(output: SharedOutput, persona: Arc<Persona>) -> Self {
        let mut conn = Self::new_without_output(persona);
        conn.output = Some(output);
        conn
    }

    pub fn load_persona_history(persona_name: &str) -> Result<ConversationHistory, Box<dyn std::error::Error>> {
        let path = format!("personas/{}/history/{}_history.json", &persona_name, persona_name);
        let content = std::fs::read_to_string(path)?;
        let history: ConversationHistory = serde_json::from_str(&content)?;
        Ok(history)
    }

    pub fn save_persona_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = std::fs::create_dir_all(format!("personas/{}/history", self.persona.name));

        let limit = self.persona.history_message_limit;

        let recent_start = if self.local_history.len() > limit + 1 {
            self.local_history.len() - limit
        } else {
            1
        };

        let recent_messages: Vec<Message> = self.local_history[recent_start..]
            .to_vec();

        let existing_summary = self.local_history.iter()
            .find(|msg| msg.role == "system" && msg.content.contains("[Previous conversation summary:"))
            .and_then(|msg| {
                msg.content
                    .strip_prefix("[Previous conversation summary: ")
                    .and_then(|s| s.strip_suffix("]"))
                    .map(|s| s.to_string())
            });

        let history = ConversationHistory {
            persona_name: self.persona.name.clone(),
            summary: existing_summary,
            recent_messages,
            total_message_count: self.local_history.len() - 1,
            last_updated: chrono::Utc::now().to_rfc3339(),
            summarization_count: 0,
        };

        let json = serde_json::to_string_pretty(&history)?;
        let path = format!("personas/{}/history/{}_history.json", &self.persona.name, self.persona.name);
        std::fs::write(path, json)?;

        log_info!("Saved history for {}", self.persona.name);
        Ok(())
    }

    pub fn clear_request_input(&mut self) {
        self.request.input.clear();
    }
    pub fn set_last_response_id(&mut self, id: String) {
        self.last_response_id = Some(id);
    }

    /// # save_history
    ///
    /// **Purpose:**
    /// Persists the complete conversation history to a JSON file.
    ///
    /// **Parameters:**
    /// - `path`: File path where history will be saved
    ///
    /// **Returns:**
    /// `Result<(), std::io::Error>` - Success or I/O error
    ///
    /// **Errors / Failures:**
    /// - File write permissions issues
    /// - Serialization failures (unlikely with Message struct)
    ///
    /// **Examples:**
    /// ```rust
    /// shadow.save_history("conversation_history.json")?;
    /// ```
    pub fn save_history(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self.local_history)?;
        std::fs::write(path, json)?;
        if let Some(ref _output) = self.output {
            _output.display(format!("Saved history ({} messages)", self.local_history.len()));
        } else {
            log_info!("Saved history ({} messages)", self.local_history.len());
        }
        Ok(())
    }

    /// # add_user_message
    ///
    /// **Purpose:**
    /// Adds a user message to the conversation history and prepares it for API submission.
    ///
    /// **Parameters:**
    /// - `content`: The message text from the user
    ///
    /// **Returns:**
    /// None (mutates internal state)
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    ///
    /// **Examples:**
    /// ```rust
    /// shadow.add_user_message("Tell me about Rust");
    /// ```
    pub fn add_user_message(&mut self, content: &str) {
        log_info!("Adding user message: {}", content);

        let new_msg = Message {
            role: "user".to_string(),
            content: content.to_string(),
        };
        self.local_history.push(new_msg.clone());

        if self.last_response_id.is_none() {
            // No response ID - send full history (system prompt + all messages)
            self.request.input = self.local_history.clone();
        } else {
            // Have response ID - only send new message
            self.request.input = vec![new_msg.clone()];
        }

    }

    /// # handle_response
    ///
    /// **Purpose:**
    /// Sends the current request to the Grok API and processes the response.
    ///
    /// **Parameters:**
    /// None (uses internal request state)
    ///
    /// **Returns:**
    /// `Result<(), Box<dyn std::error::Error>>` - Success or propagated error
    ///
    /// **Errors / Failures:**
    /// - Network connectivity issues
    /// - API authentication failures
    /// - Rate limiting or quota exceeded
    /// - Malformed API responses
    /// - JSON deserialization errors
    ///
    /// **Examples:**
    /// ```rust
    /// shadow.add_user_message("Hello");
    /// shadow.handle_response().await?;
    /// ```
    pub async fn handle_response_streaming(
        &mut self,
        tx: mpsc::UnboundedSender<StreamChunk>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info!("Handling Grok API response");
        
        self.request.previous_response_id = self.last_response_id.clone();

        let response = self.client
            .post("https://api.x.ai/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&self.request)
            .send()
            .await?;
        log_info!("{:?}", self.request);

        let status = response.status();

        if !status.is_success() {
            log_error!("Failed to get response");
            let _text = response.text().await?;
            tx.send(StreamChunk::Error(format!("API Error: {}", status)))?;
            return Ok(());
        }
        log_info!("API Response received.");

        let mut stream = response.bytes_stream();
        let mut full_reply = String::new();
        let mut response_id:  Option<String> = None;
        let mut line_buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk_bytes = chunk_result?;
            line_buffer.push_str(&String::from_utf8_lossy(&chunk_bytes));

            // Process only complete lines (ending in newline)
            while let Some(newline_pos) = line_buffer.find('\n') {
                let line = line_buffer[..newline_pos].to_string();
                line_buffer.drain(..=newline_pos);

                if let Some(data) = line.strip_prefix("data: ") {

                    if let Ok(delta) = serde_json::from_str::<DeltaChunk>(data) {
                        if delta.type_ == "response.output_text.delta" {
                            full_reply.push_str(&delta.delta);

                            tx.send(StreamChunk::Delta(delta.delta))?;
                        }
                    }

                    if let Ok(completed) = serde_json::from_str::<CompletedChunk>(data) {
                        if completed.type_ == "response.completed" {
                            log_info!("Received completed signal: {}", completed.response.id);
                            response_id = Some(completed.response.id.clone());
                        }
                    }
                }
            }
        }
        log_info!("Stream ended, response_id: {:?}", response_id);

        if !full_reply.is_empty() {
            self.local_history.push(Message {
                role: "assistant".to_string(),
                content: full_reply.clone(),
            });
            self.request.input.clear();
        }

        if self.persona.enable_history {
            if let Err(e) = self.save_persona_history() {
                log_error!("Failed to save history: {}", e);
            }

            if self.should_summarize() {
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

        if let Some(id) = response_id {
            self.last_response_id = Some(id.clone());
            tx.send(StreamChunk::Complete{
                response_id: id,
                full_reply: full_reply.clone(),
            })?;
        }

        Ok(())
    }

    pub fn should_summarize(&self) -> bool {
        if !self.persona.enable_history {
            return false;
        }

        let message_count = self.local_history.iter()
            .filter(|msg| msg.role != "system" || !msg.content.contains("[Previous conversation summary:"))
            .count();

        message_count > self.persona.summary_threshold
    }

    pub fn extract_summary_from_response(&self, response_text: &str) -> Result<String, Box<dyn std::error::Error>> {
        for line in response_text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(completed) = serde_json::from_str::<CompletedChunk>(data) {
                    if completed.type_ == "response.completed" {
                        if let Some(output) = completed.response.output.first() {
                            if let Some(content) = output.content.first() {
                                return Ok(content.text.clone());
                            }
                        }
                    }
                }

                if let Ok(delta) = serde_json::from_str::<DeltaChunk>(data) {
                    if delta.type_ == "response.output_text.delta" {
                        return Ok(delta.delta);
                    }
                }
            }
        }

        Err("Failed to extract summary from response".into())
    }

    pub fn archive_full_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all("personas/archives")?;
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let path = format!("personas/archives/{}_{}.json", self.persona.name, timestamp);

        let json = serde_json::to_string_pretty(&self.local_history)?;
        std::fs::write(path, json)?;

        log_info!("Archived full history for {}", self.persona.name);
        Ok(())
    }

    pub async fn summarize_history(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        let historian_path = "personas/historian/historian.yaml";
        let historian = match Persona::from_yaml_file(Path::new(historian_path)) {
            Ok(p) => Arc::new(p),
            Err(e) => {
                return Err(format!("Failed to load historian persona: {}", e).into());
            }
        };

        let limit = self.persona.history_message_limit;
        let cutoff_index = if self.local_history.len() > limit + 1 {
            self.local_history.len() - limit
        } else {
            return Ok(());
        };

        let messages_to_summarize = &self.local_history[1..cutoff_index];
        if messages_to_summarize.is_empty() {
            return Ok(());
        }

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

        let response = self.client
            .post("https://api.x.ai/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&summary_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Summarization API error: {}", response.status()).into());
        }

        let response_text = response.text().await?;

        let summary = self.extract_summary_from_response(&response_text)?;

        log_info!("Summary generated: {}", summary);

        let system_prompt = self.local_history[0].clone();
        let summary_message = Message {
            role: "system".to_string(),
            content: format!("[Previous conversation summary: {}", summary),
        };

        let recent_messages = self.local_history[cutoff_index..].to_vec();

        self.archive_full_history()?;

        self.local_history = vec![system_prompt, summary_message];
        self.local_history.extend(recent_messages);

        log_info!("History rebuilt with summary. Messages: {} -> {}",
            cutoff_index + limit, self.local_history.len());

        Ok(())
    }

    /// # handle_response
    ///
    /// **Purpose:**
    /// Blocking response handler for CLI mode. Sends request and displays output synchronously.
    ///
    /// **Parameters:**
    /// None (uses internal state and output handler)
    ///
    /// **Returns:**
    /// `Result<(), Box<dyn std::error::Error>>` - Success or error
    ///
    /// **Examples:**
    /// ```rust
    /// shadow.add_user_message("Hello");
    /// shadow.handle_response().await?;
    /// ```
    pub async fn handle_response(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log_info!("Handling Grok API response (blocking mode)");
        
        self.request.previous_response_id = self.last_response_id.clone();

        let response = self.client
            .post("https://api.x.ai/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&self.request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let _text = response.text().await?;
            if let Some(ref output) = self.output {
                output.display(format!("API Error: {}", status));
            }
            return Err(format!("API Error: {}", status).into());
        }

        let mut stream = response.bytes_stream();
        let mut full_reply = String::new();
        let mut response_id: Option<String> = None;

        while let Some(chunk_result) = stream.next().await {
            let chunk_bytes = chunk_result?;
            let text = String::from_utf8_lossy(&chunk_bytes);

            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        break;
                    }

                    if let Ok(delta) = serde_json::from_str::<DeltaChunk>(data) {
                        if delta.type_ == "response.output_text.delta" {
                            full_reply.push_str(&delta.delta);
                            
                            // Display incrementally for CLI
                            if self.output.is_some() || true {
                                print!("{}", delta.delta);
                                io::stdout().flush().ok();
                            }
                        }
                    }

                    if let Ok(completed) = serde_json::from_str::<CompletedChunk>(data) {
                        if completed.type_ == "response.completed" {
                            response_id = Some(completed.response.id.clone());
                        }
                    }
                }
            }
        }

        if let Some(ref output) = self.output {
            output.display("\n".to_string()); // Newline after streaming
        }

        if !full_reply.is_empty() {
            self.local_history.push(Message {
                role: "assistant".to_string(),
                content: full_reply.clone(),
            });
            self.request.input.clear();
        }

        if let Some(id) = response_id {
            self.last_response_id = Some(id);
        }

        Ok(())
    }
}