//! # Daegonica Module: persona
//!
//! **Purpose:** Persona management and agent spawning system
//!
//! **Context:**
//! - Loads persona configurations from YAML files
//! - Manages multiple concurrent agents with different personalities
//! - (Note: Currently under development, not fully integrated)
//!
//! **Responsibilities:**
//! - Define persona data structures
//! - Load personas from configuration files
//! - Manage agent lifecycle and message routing
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::prelude::*;

/// # Persona
///
/// **Summary:**
/// Configuration for an AI agent personality with custom prompts and settings.
///
/// **Fields:**
/// - `name`: Display name of the persona
/// - `description`: Optional description of the persona's purpose
/// - `system_prompt`: The system prompt that defines the persona's behavior
/// - `temperature`: Optional temperature setting for response randomness
/// - `max_tokens`: Optional maximum token limit for responses
/// - `tools`: Optional list of available tools for this persona
/// - `memory_policy`: Optional memory management strategy
/// - `startup_commands`: Optional commands to run on agent startup
///
/// **Usage Example:**
/// ```rust
/// let persona = Persona::from_yaml_file(Path::new("personas/shadow.yaml"))?;
/// println!("Loaded: {}", persona.name);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub system_prompt: String,

    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,

    pub description: Option<String>,
    pub tools: Option<Vec<String>>,

    #[serde(default = "default_true")]
    pub enable_history: bool,

    #[serde(default = "default_message_limit")]
    pub history_message_limit: usize,

    #[serde(default = "default_summary_threshold")]
    pub summary_threshold: usize,

    #[serde(default = "default_api_provider")]
    pub api_provider: String,
}

impl Persona {
    /// # from_yaml_file
    ///
    /// **Purpose:**
    /// Loads a persona configuration from a YAML file.
    ///
    /// **Parameters:**
    /// - `path`: Path to the YAML configuration file
    ///
    /// **Returns:**
    /// `anyhow::Result<Self>` - Loaded persona or error
    ///
    /// **Errors / Failures:**
    /// - File not found
    /// - Invalid YAML format
    /// - Missing required fields
    ///
    /// **Examples:**
    /// ```rust
    /// let persona = Persona::from_yaml_file(Path::new("shadow.yaml"))?;
    /// ```
    pub fn from_yaml_file(path: &Path) -> anyhow::Result<Self> {
        let s = fs::read_to_string(path)?;
        let p: Persona = serde_yaml::from_str(&s)?;
        Ok(p)
    }

}

fn default_true() -> bool { GLOBAL_CONFIG.history.enabled }
fn default_message_limit() -> usize { GLOBAL_CONFIG.history.messages_to_keep_after_summary }
fn default_summary_threshold() -> usize { GLOBAL_CONFIG.history.max_messages_before_summary }
fn default_api_provider() -> String { "grok".to_string() }

/// # PersonaRef
///
/// **Summary:**
/// Thread-safe reference-counted pointer to a Persona for sharing across threads.
///
/// **Usage Example:**
/// ```rust
/// let persona_ref: PersonaRef = Arc::new(persona);
/// ```
pub type PersonaRef = Arc<Persona>;

pub mod agent_registry;

pub use agent_registry::*;


/// Discover all available personas by scanning the personas directory
///
/// # How it works
/// - Walks through `personas/` directory recursively
/// - Finds all `.yaml` files
/// - Extracts persona name from directory structure
///
/// # Returns
/// Vector of (persona_name, yaml_path) tuples
///
/// # Example
/// ```
/// personas/
///   shadow/
///     shadow.yaml      -> ("shadow", "personas/shadow/shadow.yaml")
///   friday/
///     friday.yaml      -> ("friday", "personas/friday/friday.yaml")
///   custom/
///     my_persona.yaml  -> ("custom/my_persona", "personas/custom/my_persona.yaml")
/// ```
pub fn discover_personas() -> Result<Vec<(String, PathBuf)>, ShadowError> {
    let personas_dir = "personas";
    let mut found_personas = Vec::new();

    if !std::path::Path::new(personas_dir).exists() {
        return Err(ShadowError::IoError("personas/ directory not found".to_string()));
    }

    for entry in WalkDir::new(personas_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            if let Some(parent) = path.parent() {
                if let Some(dir_name) = parent.file_name() {
                    let persona_name = dir_name.to_string_lossy().to_string();
                    found_personas.push((persona_name, path.to_path_buf()));
                }
            }
        }
    }

    found_personas.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(found_personas)
}

pub fn get_default_persona() -> Result<String, ShadowError> {
    let personas = discover_personas()?;

    if personas.iter().any(|(name, _)| name == "shadow") {
        return Ok("shadow".to_string());
    }

    personas.first()
        .map(|(name, _)| name.clone())
        .ok_or(ShadowError::IoError("No personas found".to_string()))
}