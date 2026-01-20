//! # Daegonica Module: models
//!
//! **Purpose:** Core data structures for API communication and application state
//!
//! **Context:**
//! - Used by Grok API client for request/response handling
//! - Defines message formats, error structures, and user actions
//! - Shared across TUI and CLI modes
//!
//! **Responsibilities:**
//! - Define serializable structures for Grok API interaction
//! - Model conversation messages and chat requests
//! - Handle API error responses
//! - Define user input action types
//! - Does NOT contain business logic (pure data structures)
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use serde::{Serialize, Deserialize};

// Response handling
/// # Message
///
/// **Summary:**
/// Represents a single message in a conversation with role and content.
///
/// **Fields:**
/// - `role`: The role of the message sender ("user", "assistant", "system")
/// - `content`: The actual text content of the message
///
/// **Usage Example:**
/// ```rust
/// let msg = Message {
///     role: "user".to_string(),
///     content: "Hello Shadow!".to_string(),
/// };
/// ```
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}
/// # ChatRequest
///
/// **Summary:**
/// Request payload for the Grok API chat endpoint with conversation context.
///
/// **Fields:**
/// - `model`: The Grok model to use (e.g., "grok-4-fast")
/// - `input`: Vector of messages forming the conversation history
/// - `temperature`: Sampling temperature for response randomness (0.0-1.0)
/// - `previous_response_id`: Optional ID for conversation continuity
///
/// **Usage Example:**
/// ```rust
/// let request = ChatRequest {
///     model: "grok-4-fast".to_string(),
///     input: vec![msg],
///     temperature: 0.7,
///     previous_response_id: None,
/// };
/// ```
#[derive(Serialize, Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub input: Vec<Message>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    pub stream: bool,
}
#[derive(Debug, Deserialize)]
pub struct DeltaChunk {
    #[serde(rename = "type")]
    pub type_: String,  // "response.output_text.delta"
    pub delta: String,   // "Test", " acknowledged", etc.
    pub sequence_number: u32,
    pub content_index: u32,
    pub item_id: String,
    pub output_index: u32,
}
#[derive(Debug, Deserialize)]
pub struct CompletedChunk {
    #[serde(rename = "type")]
    pub type_: String,  // "response.completed"
    pub response: ResponsesApiResponse,  // Full response with ID
}

/// # StreamChunk
///
/// **Summary:**
/// Enum for typed communication between background streaming task and main TUI thread.
///
/// **Variants:**
/// - `Delta(String)`: Incremental text chunk from SSE stream
/// - `Complete(String)`: Final complete response text
/// - `Error(String)`: Error message from streaming failure
///
/// **Usage Example:**
/// ```rust
/// tx.send(StreamChunk::Delta("Hello".to_string()))?;
/// tx.send(StreamChunk::Complete("Full response".to_string()))?;
/// ```
#[derive(Debug, Clone)]
pub enum StreamChunk {
    Delta(String),
    Complete{
        response_id: String,
        full_reply: String,
    },
    Error(String),
    Info(String),
}

/// # ResponsesApiResponse
///
/// **Summary:**
/// Complete response from the Grok API /v1/responses endpoint.
///
/// **Fields:**
/// - `id`: Unique identifier for this response
/// - `object`: Object type returned by the API
/// - `created_at`: Unix timestamp of response creation
/// - `model`: The model that generated the response
/// - `output`: Vector of output messages from the assistant
/// - `usage`: Optional token usage statistics
///
/// **Usage Example:**
/// ```rust
/// let response: ResponsesApiResponse = serde_json::from_str(&json_text)?;
/// ```
#[derive(Deserialize, Debug)]
pub struct ResponsesApiResponse{
    pub id: String,
    pub object: String,
    #[serde(rename = "created_at")]
    pub created_at: u64,
    pub model: String,
    pub output: Vec<OutputMessage>,
    #[serde(default)]
    pub usage: Option<Usage>,
}
/// # OutputMessage
///
/// **Summary:**
/// Individual message within the API response output array.
///
/// **Fields:**
/// - `id`: Unique identifier for this output message
/// - `role`: Role of the message sender (typically "assistant")
/// - `type_`: Type of the message
/// - `status`: Processing status of the message
/// - `content`: Vector of content blocks containing the actual response
///
/// **Usage Example:**
/// ```rust
/// if let Some(first_msg) = response.output.first() {
///     println!("Status: {}", first_msg.status);
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct OutputMessage {
    pub id: String,
    pub role: String,
    #[serde(rename= "type")]
    pub type_: String,
    pub status: String,
    pub content: Vec<ContentBlock>,
}
/// # ContentBlock
///
/// **Summary:**
/// Individual content block within an output message containing actual text.
///
/// **Fields:**
/// - `type_`: Type of content (typically "output_text")
/// - `text`: The actual text content from the assistant
///
/// **Usage Example:**
/// ```rust
/// for block in message.content.iter() {
///     if block.type_ == "output_text" {
///         println!("{}", block.text);
///     }
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String
}
/// # Usage
///
/// **Summary:**
/// Token usage statistics for API requests and responses.
///
/// **Fields:**
/// - `input_tokens`: Number of tokens in the input/request
/// - `output_tokens`: Number of tokens in the output/response
/// - `total_tokens`: Total tokens used (input + output)
///
/// **Usage Example:**
/// ```rust
/// if let Some(usage) = response.usage {
///     println!("Tokens used: {}", usage.total_tokens);
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}



// Error response from the API
/// # ApiErrorDetail
///
/// **Summary:**
/// Detailed error information from the Grok API when requests fail.
///
/// **Fields:**
/// - `message`: Human-readable error message
/// - `error_type`: Optional error type classification
/// - `code`: Optional error code
///
/// **Usage Example:**
/// ```rust
/// println!("Error: {}", error.message);
/// ```
#[derive(Deserialize, Debug)]
pub struct ApiErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<String>,
}
/// # ApiErrorResponse
///
/// **Summary:**
/// Top-level error response wrapper from the Grok API.
///
/// **Fields:**
/// - `error`: The detailed error information
///
/// **Usage Example:**
/// ```rust
/// let error_response: ApiErrorResponse = serde_json::from_str(&error_text)?;
/// println!("API Error: {}", error_response.error.message);
/// ```
#[derive(Deserialize, Debug)]
pub struct ApiErrorResponse {
    pub error: ApiErrorDetail,
}
/// # InputAction
///
/// **Summary:**
/// Enum representing all possible actions that can result from user input.
///
/// **Variants:**
/// - `Quit`: Exit the application
/// - `DoNothing`: No action needed (e.g., invalid input handled)
/// - `ContinueNoSend(String)`: Display a message without sending to API
/// - `SendAsMessage(String)`: Send the message to the Grok API
/// - `PostTweet(String)`: Post content to Twitter
/// - `DraftTweet(String)`: Generate a tweet draft via AI
/// - `NewAgent(String)`: Create a new agent with specified persona
/// - `CloseAgent`: Close the current agent
/// - `ListAgents`: Display all active agents
#[derive(Debug)]
pub enum InputAction {
    Quit,
    DoNothing,

    // Commands that result in a message to be displayed but not sent
    ContinueNoSend(String),

    // Send message to Grok API
    SendAsMessage(String),
    ClearHistory,
    HistoryInfo,
    SaveHistory,
    Summarize,

    // Twitter-related actions
    PostTweet(String),
    DraftTweet(String),

    // Agent management actions
    NewAgent(String),
    AgentStatus,
    CloseAgent,
    ListAgents,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConversationHistory {
    pub persona_name: String,
    pub summary: Option<String>,
    pub recent_messages: Vec<Message>,
    pub total_message_count: usize,
    pub last_updated: String,
    pub summarization_count: usize,
}

impl ConversationHistory {
    pub fn new(persona_name: String) -> Self {
        Self {
            persona_name,
            summary: None,
            recent_messages: Vec::new(),
            total_message_count: 0,
            last_updated: chrono::Utc::now().to_rfc3339(),
            summarization_count: 0,
        }
    }
}