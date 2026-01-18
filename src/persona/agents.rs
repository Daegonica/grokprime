//! # Daegonica Module: persona::agents
//!
//! **Purpose:** Agent spawning and lifecycle management
//!
//! **Context:**
//! - Creates and manages individual agent tasks
//! - Provides message passing infrastructure
//! - (Note: Currently under development, not fully integrated)
//!
//! **Responsibilities:**
//! - Spawn agent tasks with persona configuration
//! - Handle agent message routing
//! - Manage agent shutdown
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use crate::persona::PersonaRef;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// # AgentHandle
///
/// **Summary:**
/// Handle for communicating with a running agent task.
///
/// **Fields:**
/// - `id`: Unique identifier for this agent
/// - `tx`: Message sender channel to the agent
/// - `shutdown`: Oneshot channel for shutdown signal
///
/// **Usage Example:**
/// ```rust
/// let handle = start_agent(persona).await;
/// handle.tx.send("Hello".to_string()).await?;
/// ```
pub struct AgentHandle {
    pub id: Uuid,
    pub tx: mpsc::Sender<String>,
    pub shutdown: oneshot::Sender<()>,
}

/// # start_agent
///
/// **Purpose:**
/// Spawns a new agent task with the specified persona configuration.
///
/// **Parameters:**
/// - `persona`: The persona configuration for this agent
///
/// **Returns:**
/// `AgentHandle` - Handle for communicating with the spawned agent
///
/// **Errors / Failures:**
/// - None (spawns successfully, errors handled within task)
///
/// **Examples:**
/// ```rust
/// let persona = Arc::new(persona_config);
/// let handle = start_agent(persona).await;
/// ```
pub async fn start_agent(persona: PersonaRef) -> AgentHandle {
    let (tx, mut rx) = mpsc::channel::<String>(32);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    let id = Uuid::new_v4();
    let persona_clone = persona.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(msg) = rx.recv() => {
                    println!("[{}] {}: {}", persona_clone.name, id, msg);
                }
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
    });
    AgentHandle { id, tx, shutdown: shutdown_tx }
}