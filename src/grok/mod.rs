//! # Daegonica Module: grok
//!
//! **Purpose:** Grok AI API integration modules
//!
//! **Context:**
//! - Core modules for interacting with x.ai Grok API
//! - Separated into client, conversation, and history layers
//!
//! **Responsibilities:**
//! - Expose all Grok-related modules
//! - Provide unified interface through GrokConnection
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

pub mod agent;
pub mod client;
pub mod conversations;
pub mod history;