use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::fs;
use std::path::Path;

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
    pub fn from_yaml_file(path: &Path) -> anyhow::Result<Self> {
        let s = fs::read_to_string(path)?;
        let p: Persona = serde_yaml::from_str(&s)?;
        Ok(p)
    }
}
pub type PersonaRef = Arc<Persona>;