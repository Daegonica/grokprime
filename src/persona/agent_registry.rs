//! # Daegonica Module: persona::agent_registry
//!
//! **Purpose:** Central registry for managing multiple concurrent agents
//!
//! **Context:**
//! - Tracks all active agent instances
//! - Routes messages to specific agents
//! - (Note: Currently under development, not fully integrated)
//!
//! **Responsibilities:**
//! - Start new agents with personas
//! - List all active agents
//! - Route messages to agents by ID
//! - Stop agents gracefully
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use crate::persona::PersonaRef;
use crate::persona::agents::{AgentHandle, start_agent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// # AgentRegistry
///
/// **Summary:**
/// Thread-safe registry for managing multiple concurrent agent instances.
///
/// **Fields:**
/// - `agents`: Map of agent UUIDs to their handles (protected by RwLock)
///
/// **Usage Example:**
/// ```rust
/// let registry = AgentRegistry::new();
/// let id = registry.start_agent(persona_ref).await;
/// registry.send_message(id, "Hello".to_string()).await;
/// ```
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<Uuid, AgentHandle>>>,
}

impl AgentRegistry {
    /// # new
    ///
    /// **Purpose:**
    /// Creates a new empty agent registry.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// Initialized AgentRegistry
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// # start_agent
    ///
    /// **Purpose:**
    /// Starts a new agent with the given persona and registers it.
    ///
    /// **Parameters:**
    /// - `persona`: The persona configuration for the agent
    ///
    /// **Returns:**
    /// UUID of the newly started agent
    ///
    /// **Errors / Failures:**
    /// - None (agent task spawn failures handled internally)
    pub async fn start_agent(&self, persona: PersonaRef) -> Uuid {
        let handle = start_agent(persona).await;
        let id = handle.id;
        self.agents.write().await.insert(id, handle);
        id
    }

    /// # list_agents
    ///
    /// **Purpose:**
    /// Returns a list of all currently registered agent IDs.
    ///
    /// **Parameters:**
    /// None
    ///
    /// **Returns:**
    /// Vector of agent UUIDs
    ///
    /// **Errors / Failures:**
    /// - None (infallible)
    pub async fn list_agents(&self) -> Vec<Uuid> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// # send_message
    ///
    /// **Purpose:**
    /// Sends a message to a specific agent by ID.
    ///
    /// **Parameters:**
    /// - `id`: The agent UUID
    /// - `msg`: The message to send
    ///
    /// **Returns:**
    /// `bool` - true if message sent successfully, false if agent not found
    ///
    /// **Errors / Failures:**
    /// - Returns false if agent ID doesn't exist
    /// - Returns false if channel is closed
    pub async fn send_message(&self, id: Uuid, msg: String) -> bool {
        let agents = self.agents.read().await;
        if let Some(agent) = agents.get(&id) {
            agent.tx.send(msg).await.is_ok()
        } else {
            false
        }
    }

    /// # stop_agent
    ///
    /// **Purpose:**
    /// Stops an agent and removes it from the registry.
    ///
    /// **Parameters:**
    /// - `id`: The agent UUID to stop
    ///
    /// **Returns:**
    /// `bool` - true if agent was stopped, false if not found
    ///
    /// **Errors / Failures:**
    /// - Returns false if agent ID doesn't exist
    pub async fn stop_agent(&self, id: Uuid) -> bool {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.remove(&id) {
            let _ = agent.shutdown.send(());
            true
        } else {
            false
        }
    }
}