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
use std::path::Path;

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
    pub description: Option<String>,
    pub system_prompt: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<String>>,
    pub memory_policy: Option<String>,
    pub startup_commands: Option<Vec<String>>,
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

pub mod agents;
pub mod agent_registry;

pub use agents::*;
pub use agent_registry::*;