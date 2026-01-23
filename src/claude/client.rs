//! # Daegonica Module: claude::client
//!
//! **Purpose:** Claude API communication layer
//!
//! **Context:**
//! - Handles HTTP communication with Anthropic Claude API
//! - Implements LlmClient trait for integration
//!
//! **Responsibilities:**
//! - Authenticate with x-api-key header
//! - Send requests to Claude /v1/messages endpoint
//! - Stream SSE responses
//! - Parse Claude-specific event format
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-21

use crate::prelude::*;
use crate::llm::{LlmClient, StreamResponse};
use crate::claude::models::*;
use futures_util::StreamExt;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ClaudeClient {
    api_key: String,
    client: Client,
}

impl ClaudeClient {
    pub fn new() -> Result<Self, String> {
        dotenv().ok();
        let api_key = env::var("CLAUDE_KEY")
            .map_err(|_| "CLAUDE_KEY environment variable not set".to_string())?;

        Ok( ClaudeClient {
            api_key,
            client: Client::new(),
        })
    }

    /// Convert generic ChatRequest to Claude-specific format
    ///
    /// # Key Differences:
    /// - Extract system prompt from messages[0]
    /// - Filter out system message from messages array
    /// - Ensure max_tokens is set (required by Claude)
    fn adapt_request(&self, request: &ChatRequest) -> ClaudeRequest {
        let system = request.input.iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let messages: Vec<ClaudeMessage> = request.input.iter()
            .filter(|m| m.role != "system")
            .map(|m| ClaudeMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            system,
            messages,
            temperature: Some(request.temperature),
            stream: true,
        }
    }

}

#[async_trait]
impl LlmClient for ClaudeClient {
    async fn send_streaming(
        &self,
        request: &ChatRequest,
        tx: mpsc::UnboundedSender<StreamChunk>,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>> {

        let claude_request = self.adapt_request(request);

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&claude_request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await?;
            log_error!("Claude API error: {} - {}", status, error_text);
            tx.send(StreamChunk::Error(format!("API error: {} - {}", status, error_text)))?;
            return Err(format!("API error: {}", status).into());
        }


        let mut stream = response.bytes_stream();
        let mut full_reply = String::new();
        let mut response_id: Option<String> = None;
        let mut line_buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk_bytes = chunk_result?;
            line_buffer.push_str(&String::from_utf8_lossy(&chunk_bytes));

            while let Some(newline_pos) = line_buffer.find('\n') {
                let line = line_buffer[..newline_pos].to_string();
                line_buffer.drain(..=newline_pos);

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(msg_start) = serde_json::from_str::<ClaudeMessageStart>(data) {
                        if msg_start.type_ == "message_start" {
                            response_id = Some(msg_start.message.id.clone());
                        }
                    }

                    if let Ok(content_delta) = serde_json::from_str::<ClaudeContentDelta>(data) {
                        if content_delta.type_ == "content_block_delta" {
                            let text = &content_delta.delta.text;
                            full_reply.push_str(text);
                            tx.send(StreamChunk::Delta(text.clone()))?;
                        }
                    }
                }
            }
        }


        Ok(StreamResponse {
            response_id: response_id.ok_or("No response ID received")?,
            full_text: full_reply,
        })
    }

    async fn send_blocking(
        &self,
        _request: &ChatRequest,
        _print_stream: bool,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>> {
        unimplemented!("Claude send_blocking not yet implemented")
    }
}