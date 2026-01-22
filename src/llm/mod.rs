//! # Daegonica Module: llm
//!
//! **Purpose:** Unified trait for multiple LLM API clients
//!
//! **Context:**
//! - Defines common behavior for Grok, Claude, and future APIs
//! - Enables generic Connection that works with any LLM
//!
//! **Responsibilities:**
//! - Define LlmClient trait for API communication
//! - Define shared response types
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-21

use crate::prelude::*;
use async_trait::async_trait;

/// # StreamResponse
///
/// **Summary:**
/// Unified response structure for all LLM clients.
///
/// **Fields:**
/// - `response_id`: API-specific ID for conversation continuity
/// - `full_text`: Complete assembled response text
pub struct StreamResponse {
    pub response_id: String,
    pub full_text: String,
}

/// # LlmClient
///
/// **Summary:**
/// Trait defining the contract for LLM API clients.
///
/// **Purpose:**
/// All LLM clients (Grok, Claude, etc.) must implement this trait
/// to work with the generic Connection struct.
///
/// **Required Methods:**
/// - `send_streaming`: Send request and stream response chunks
/// - `send_blocking`: Send request and return complete response
///
/// **Design Notes:**
/// - Uses async_trait macro for async trait methods (required in Rust)
/// - All errors boxed for consistency
/// - Request uses generic ChatRequest (must be adapted by implementer)
#[async_trait]
pub trait LlmClient: Send + Sync + Clone {
    /// Send a chat request and stream response chunks via channel
    ///
    /// # Parameters
    /// - `request`: The chat request payload
    /// - `tx`: Channel for sending StreamChunk updates
    ///
    /// # Returns
    /// Complete StreamResponse with response_id and full_text
    ///
    /// # Errors
    /// - Network failures
    /// - Authentication errors
    /// - API errors (non-2xx status)
    /// - Parsing errors
    async fn send_streaming(
        &self,
        request: &ChatRequest,
        tx: mpsc::UnboundedSender<StreamChunk>,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>>;

    /// Send a chat request and return complete response (for CLI mode)
    ///
    /// # Parameters
    /// - `request`: The chat request payload
    /// - `print_stream`: Whether to print chunks to stdout
    ///
    /// # Returns
    /// Complete StreamResponse with response_id and full_text
    async fn send_blocking(
        &self,
        request: &ChatRequest,
        print_stream: bool,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>>;
}

pub mod client;

#[derive(Debug, Clone)]
pub enum AnyClient {
    Grok(GrokClient),
    Claude(ClaudeClient),
}

#[async_trait]
impl LlmClient for AnyClient {
    async fn send_streaming(
        &self,
        request: &ChatRequest,
        tx: mpsc::UnboundedSender<StreamChunk>,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>> {
        match self {
            AnyClient::Grok(client) => client.send_streaming(request, tx).await,
            AnyClient::Claude(client) => client.send_streaming(request, tx).await,
        }
    }

    async fn send_blocking(
        &self,
        request: &ChatRequest,
        print_stream: bool,
    ) -> Result<StreamResponse, Box<dyn std::error::Error>> {
        match self {
            AnyClient::Grok(client) => client.send_blocking(request, print_stream).await,
            AnyClient::Claude(client) => client.send_blocking(request, print_stream).await,
        }
    }
}