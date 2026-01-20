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

pub use dlog::{log_init, log_error, log_info, enums::OutputTarget};

pub use crate::models::*;
pub use crate::capitalize_first;

pub use crate::utilities::outputs::{
    OutputHandler, 
    SharedOutput, 
    CliOutput,
};
pub use crate::utilities::cli::Args;

pub use crate::twitter::*;
pub use crate::persona::{Persona, PersonaRef};

pub use crate::grok::agent::GrokConnection;

pub use crate::user::user_input::UserInput;
pub use crate::user::system_info::OsInfo;

pub use crate::tui::{ShadowApp, AgentPane, MessageSource, UnifiedMessage};
pub use crate::config::{AppConfig, GrokConfig, TuiConfig, HistoryConfig, GLOBAL_CONFIG};