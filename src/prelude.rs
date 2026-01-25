//! # Daegonica Module: prelude
//!
//! **Purpose:** Centralized import prelude for common types and traits
//!
//! **Context:**
//! - Used throughout the codebase via `use crate::prelude::*`
//! - Reduces boilerplate imports across modules
//!
//! **Responsibilities:**
//! - Re-export commonly used external crate types and traits
//! - Re-export internal types and modules for convenience
//! - Provide a single import point for frequently used items
//! - Does NOT implement new functionality
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------


// Rust crates
pub use reqwest::Client;
pub use serde::{Deserialize, Serialize};
pub use serde_json;
pub use dotenv::dotenv;

pub use std::sync::Arc;
pub use std::env;
pub use std::io::{self, BufRead, Write};

pub use tokio::sync::mpsc;
pub use tokio::task;

pub use std::fs::{self, read_to_string, write, File};
pub use std::path::{Path, PathBuf};

// *** Current crate ***

// Features
pub use crate::twitter::*;

// Config file
pub use crate::config::{AppConfig, GrokConfig, TuiConfig, HistoryConfig, GLOBAL_CONFIG};

// User specific
pub use crate::user::user_input::UserInput;
pub use crate::user::system_info::OsInfo;

// Utility files
pub use crate::models::*;
pub use crate::capitalize_first;
pub use crate::errors::ShadowError;
pub use crate::utilities::cli::Args;
pub use crate::utilities::outputs::{
    OutputHandler, 
    SharedOutput, 
    CliOutput,
};

// Agent tracking
pub use crate::agent_history::conversations::GrokConversation;
pub use crate::agent_history::history::HistoryManager;
pub use crate::persona::{
    Persona,
    PersonaRef,
};
pub use crate::persona::agent_manager::AgentManager;
pub use crate::persona::agent::AgentInfo;

// AI Connections
pub use crate::grok::client::GrokClient;
pub use crate::llm::client::Connection;
pub use crate::llm::{LlmClient, StreamResponse};
pub use crate::claude::client::ClaudeClient;

// TUI related
pub use crate::tui::{ShadowApp, AgentPane, MessageSource, UnifiedMessage};

// Daegonica Software crates
pub use dlog::{log_init, log_error, log_info, enums::OutputTarget};