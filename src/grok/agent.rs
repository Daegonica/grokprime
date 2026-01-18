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
pub struct GrokConnection {
    api_key: String,
    client: Client,
    request: ChatRequest,
    last_response_id: Option<String>,
    pub local_history: Vec<Message>,
    output: SharedOutput,
}


impl GrokConnection {

    /// # new
    ///
    /// **Purpose:**
    /// Creates a new GrokConnection instance with loaded history or fresh system prompt.
    ///
    /// **Parameters:**
    /// - `output`: Shared output handler for displaying messages
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
    /// let output: SharedOutput = Arc::new(CliOutput);
    /// let shadow = GrokConnection::new(output);
    /// ```
    pub fn new(output: SharedOutput) -> Self {
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
                    output.display(format!("[INFO] Loaded {} messages from history.", loaded.len()));
                    local_history = loaded;

                }
                _ => {
                    output.display(format!("[WARNING] History file invalid or empty -> starting fresh with system prompt."));
                }
            }
        } else {
            output.display(format!("[INFO] No history file found -> starting fresh."));
        }

        let request = ChatRequest {
            model: "grok-4-fast".to_string(),
            input: local_history.clone(),
            temperature: 0.7,
            previous_response_id: None,
        };

        GrokConnection{
            api_key, 
            client: Client::new(), 
            request,
            last_response_id: None,
            local_history,
            output,
        }
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
        self.output.display(format!("Saved history ({} messages)", self.local_history.len()));
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
    pub async fn handle_response(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        
        self.request.previous_response_id = self.last_response_id.clone();

        let response = self.client
            .post("https://api.x.ai/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&self.request)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        let mut reply_opt: Option<String> = None;

        if status.is_success() {
            match serde_json::from_str::<ResponsesApiResponse>(&text) {
                Ok(res) => {
                    if let Some(first_msg) = res.output.first() {
                        if let Some(first_block) = first_msg.content.first() {
                            if first_block.type_ == "output_text" {
                                let reply = first_block.text.trim().to_string();
                                self.output.display(format!("Shadow: {}", reply));
                                reply_opt = Some(reply);
                                self.request.input.clear();
                            } else {
                                self.output.display(format!("Unexpected content type: {}", first_block.type_));
                            }
                        } else {
                            self.output.display(format!("No content blocks in output message."));
                        }
                    } else {
                        self.output.display(format!("No output messages returned."));
                    }

                    self.last_response_id = Some(res.id.clone());
                }

                Err(e) => {
                    self.output.display(format!("Failed to parse /v1/responses JSON: {}", e));
                    self.output.display(format!("Raw responses: {}", text));
                }
            }
        } else {
            match serde_json::from_str::<ApiErrorResponse>(&text) {
                Ok(error_body) => {
                    self.output.display(format!("API Error: {}", error_body.error.message));
                    if let Some(code) = error_body.error.code {
                        self.output.display(format!("Code: {}", code));
                    }
                }
                Err(_) => {
                    self.output.display(format!("Request failed with status: {}", status));
                    self.output.display(format!("Raw response: {}", text));
                }
            }
        }

        if let Some(reply) = reply_opt {
            self.local_history.push(Message {
                role: "assistant".to_string(),
                content: reply,
            });
        }

        Ok(())
    }

}