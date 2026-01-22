//! # Daegonica Module: grok::client
//!
//! **Purpose:** Pure Grok API communication layer
//!
//! **Context:**
//! - Handles all HTTP communication with x.ai Grok API
//! - Stateless - does not manage conversation history
//! - Used by GrokConnection for actual API calls
//!
//! **Responsibilities:**
//! - Authenticate API requests with bearer token
//! - Send chat requests to Grok endpoint
//! - Stream responses via Server-Sent Events (SSE)
//! - Parse response chunks into structured data
//! - Handle API Errors and status codes
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use futures_util::StreamExt;
use crate::prelude::*;
use crate::llm::{LlmClient, StreamResponse};

/// # GrokClient
///
/// **Summary:**
/// Stateless HTTP client for Grok API communication.
///
/// **Fields:**
/// - `api_key`: Bearer token for API authentication
/// - `client`: Reqwest HTTP client instance
///
/// **Usage Example:**
/// ```rust
/// let client = GrokClient::new()?;
/// let response = client.send_request(&request).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GrokClient {
    api_key: String,
    client: Client,
}

impl GrokClient {
    /// # new
    ///
    /// **Purpose:**
    /// Creates a new Grok API client with credentials from environment.
    ///
    /// **Parameters:**
    /// None (reads GROK_KEY from env)
    ///
    /// **Returns:**
    /// `Result<Self, String>` - Initialized client or error if GROK_KEY missing
    ///
    /// **Errors / Failures:**
    /// - GROK_KEY environment variable not set
    ///
    /// **Examples:**
    /// ```rust
    /// let client = GrokClient::new()?;
    /// ```
    pub fn new() -> Result<Self, String> {
        dotenv().ok();
        let api_key = env::var("GROK_KEY")
            .map_err(|_| "GROK_KEY environment variable not set".to_string())?;

        Ok(GrokClient{
            api_key,
            client: Client::new(),
        })
    }

    /// # send_streaming_request
    ///
    /// **Purpose:**
    /// Sends a chat request to Grok API and streams response chunks via channel.
    ///
    /// **Parameters:**
    /// - `request`: The chat request payload
    /// - `tx`: Channel sender for streaming chunks
    ///
    /// **Returns:**
    /// `Result<StreamResponse, Box<dyn std::error::Error>>` - Complete response data or error
    ///
    /// **StreamResponse contains:**
    /// - `response_id`: Grok's response ID for conversation continuity
    /// - `full_text`: Complete assembled response text
    ///
    /// **Errors / Failures:**
    /// - Network Errors
    /// - API authentication failures
    /// - HTTP status Errors (non-2xx)
    /// - JSON parsing Errors
    /// - Channel send failures
    ///
    /// **Examples:**
    /// ```rust
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let response = client.send_streaming_request(&request, tx).await?;
    /// ```
    pub async fn send_streaming_request(
        &self,
        request: &ChatRequest,
        tx: mpsc::UnboundedSender<StreamChunk>,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>> {
        log_info!("Sending streaming request to Grok API");

        let response = self.client
            .post("https://api.x.ai/v1/responses")
            .bearer_auth(&self.api_key)
            .json(request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await?;
            log_error!("API error: {} - {}", status, error_text);
            tx.send(StreamChunk::Error(format!("API error: {} - {}", status, error_text)))?;
            return Err(format!("API error: {}", status).into());
        }

        log_info!("API response received, streaming chunks...");

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
                    if let Ok(delta) = serde_json::from_str::<DeltaChunk>(data) {
                        if delta.type_ == "response.output_text.delta" {
                            full_reply.push_str(&delta.delta);
                            tx.send(StreamChunk::Delta(delta.delta))?;
                        }
                    }

                    if let Ok(complete) = serde_json::from_str::<CompletedChunk>(data) {
                        if complete.type_ == "response.completed" {
                            // log_info!("FULL RESPONSE DATA: {}", data);
                            log_info!("Received completion: {}", complete.response.id);
                            response_id = Some(complete.response.id.clone());
                        }
                    }
                }
            }
        }

        log_info!("Stream ended. Response ID: {:?}, Length: {}", response_id, full_reply.len());

        Ok(StreamResponse {
            response_id: response_id.ok_or("No response ID received")?,
            full_text: full_reply,
        })
    }

    /// # send_blocking_request
    ///
    /// **Purpose:**
    /// Sends request and displays response synchronously (for CLI mode).
    ///
    /// **Parameters:**
    /// - `request`: The chat request payload
    /// - `print_stream`: Whether to print chunks as they arrive
    ///
    /// **Returns:**
    /// `Result<StreamResponse, Box<dyn std::error::Error>>` - Complete response or error
    ///
    /// **Examples:**
    /// ```rust
    /// let response = client.send_blocking_request(&request, true).await?;
    /// println!("Got: {}", response.full_text);
    /// ```
    pub async fn send_blocking_request(
        &self,
        request: &ChatRequest,
        print_stream: bool,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>> {
        log_info!("Sending blocking request to Grok API");

        let response = self.client
            .post("https://api.x.ai/v1/responses")
            .bearer_auth(&self.api_key)
            .json(request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let _error_text = response.text().await?;
            log_error!("API error: {}", status);
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
                    if data.trim() == "[DONE]" {
                        continue;
                    }

                    if let Ok(delta) = serde_json::from_str::<DeltaChunk>(data) {
                        if delta.type_ == "response.output_text.delta" {
                            full_reply.push_str(&delta.delta);

                            if print_stream {
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

        if print_stream {
            println!();
        }

        Ok(StreamResponse {
            response_id: response_id.ok_or("No response ID received")?,
            full_text: full_reply,
        })
    }

}

use async_trait::async_trait;

#[async_trait]
impl LlmClient for GrokClient {
    async fn send_streaming(
        &self,
        request: &ChatRequest,
        tx: mpsc::UnboundedSender<StreamChunk>,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>> {
        self.send_streaming_request(request, tx).await
    }

    async fn send_blocking(
        &self,
        request: &ChatRequest,
        print_stream: bool,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>> {
        self.send_blocking_request(request, print_stream).await
    }
}