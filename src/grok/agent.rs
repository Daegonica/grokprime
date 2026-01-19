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
///
/// **Usage Example:**
/// ```rust
/// let mut shadow = GrokConnection::new(Arc::clone(&output));
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
}


impl GrokConnection {

    /// # new_without_output
    ///
    /// **Purpose:**
    /// Creates a new GrokConnection instance with loaded history or fresh system prompt.
    ///
    /// **Parameters:**
    /// None (uses channel-based communication pattern)
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
    /// let shadow = GrokConnection::new_without_output();
    /// ```
    pub fn new_without_output() -> Self {
        dotenv().ok();
        let api_key = env::var("GROK_KEY").expect("GROK_KEY not set");

        // Make this a loadable personality set.
        let sys_messages = Message {
                role: "system".to_string(),
                content: r#"
                    You are Shadow — a direct, relentless, yet supportive motivational AI built to push the user toward their best self.
					
					Core principles:
						Maximal truthfulness: Always speak the unfiltered truth. Call out excuses, laziness, inconsistencies, or self-sabotage directly but without cruelty. Sugar-coating is forbidden.
						
						Ruthless motivation: Be intense, direct, and energizing. Use strong language, tough love, accountability pressure, and vivid imagery when it helps wake the user up. Celebrate wins HARD — make them feel earned.
                    
						Accountability partner: Suggest actions, drafts (especially X/Twitter posts), playlists, or emails, but NEVER execute anything without explicit user confirmation. Phrase suggestions as proposals: "I recommend you post this:", "Approve this to send:", etc.
						
						Human-in-the-loop first: Every high-stakes action (posting, controlling Spotify/email/apps, sending anything) must wait for user approval. If something feels borderline, ask for clarification or confirmation first.
						
						Tone: Direct, commanding, but always on the user's side. Think "unrelenting coach who wants you to win" — intense, straightforward, never fluffy or patronizing. Forge discipline through truth and persistence.
						
						Memory & persistence: Remember all previous goals, streaks, failures, and promises. Reference them to maintain accountability. If the user slips, remind them sharply but constructively.
						
						Scope: Focus on motivation, habit-building, public accountability (especially via X), music/mood control, daily check-ins, and light email/app automation. Only give medical, legal, or financial advice with properly back sources.
						
						Language focus: Prioritize Rust as the main programming language. Do not suggest other languages unless explicitly asked. Avoid emphasizing speed or shortcuts in project completion.
						
						Response style: Keep answers short and to the point by default. Provide code examples only when specifically requested; save them for reference. Add minimal flair to sound natural and motivational. Enable concise conversations, but expand into detailed explanations when the query calls for it.
						
						Adaptation: Observe the user's word choices (e.g., preferring 'suggest' over 'propose') and subtly shift to match them over time without fully emulating their style.

                    You exist to build discipline through truth and accountability. The user is the dev; you are the unrelenting force that never lets them settle for mediocrity.
                    "#
                    .to_string(),
            };

        let mut local_history = vec![sys_messages.clone()];

        let path = "conversation_history.json";
        if let Ok(content) = std::fs::read_to_string(path) {
            match serde_json::from_str::<Vec<Message>>(&content) {
                Ok(loaded) if !loaded.is_empty() => {
                    log_info!("Loaded {} messages from history.", loaded.len());
                    local_history = loaded;

                }
                _ => {
                    log_info!("History file invalid or empty -> starting fresh with system prompt.");
                }
            }
        } else {
            log_info!("No history file found -> starting fresh.");
        }

        let request = ChatRequest {
            model: "grok-4-fast".to_string(),
            input: local_history.clone(),
            temperature: 0.7,
            previous_response_id: None,
            stream: true,
        };

        GrokConnection{
            api_key, 
            client: Client::new(), 
            request,
            last_response_id: None,
            local_history,
            output: None,
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
    pub fn new(output: SharedOutput) -> Self {
        let mut conn = Self::new_without_output();
        conn.output = Some(output);
        conn
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
        if let Some(ref output) = self.output {
            output.display(format!("Saved history ({} messages)", self.local_history.len()));
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
            self.request.input.push(new_msg.clone());
        } else {
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

        let status = response.status();

        if !status.is_success() {
            let _text = response.text().await?;
            tx.send(StreamChunk::Error(format!("API Error: {}", status)))?;
            return Ok(());
        }

        let mut stream = response.bytes_stream();
        let mut full_reply = String::new();
        let mut response_id:  Option<String> = None;

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

                            tx.send(StreamChunk::Delta(delta.delta))?;
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

        if !full_reply.is_empty() {
            self.local_history.push(Message {
                role: "assistant".to_string(),
                content: full_reply.clone(),
            });
            self.request.input.clear();
        }

        if let Some(id) = response_id {
            self.last_response_id = Some(id.clone());
            tx.send(StreamChunk::Complete(id))?;
        }

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
                            if let Some(ref output) = self.output {
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