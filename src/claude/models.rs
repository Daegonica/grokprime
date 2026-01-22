//! # Daegonica Module: claude::models
//!
//! **Purpose:** Claude API-specific request/response structures
//!
//! **Context:**
//! - Claude API has different structure than Grok
//! - Handles Claude's message format and SSE events
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-21

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub system: String,
    pub messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub stream: bool,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct ClaudeContentDelta {
    #[serde(rename = "type")]
    pub type_: String,
    pub index: u32,
    pub delta: ClaudeDelta,
}

#[derive(Deserialize, Debug)]
pub struct ClaudeDelta {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct ClaudeMessageStart {
    #[serde(rename = "type")]
    pub type_: String,
    pub message: ClaudeMessageMeta,
}

#[derive(Deserialize, Debug)]
pub struct ClaudeMessageMeta {
    pub id: String,
    pub model: String,
    pub role: String,
}