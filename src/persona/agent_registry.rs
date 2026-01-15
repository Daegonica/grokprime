use crate::persona::PersonaRef;
use crate::agent::{AgentHandle, start_agent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<Uuid, AgentHandle>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_agent(&self, persona: PersonaRef) -> Uuid {
        let handle = start_agent(persona).await;
        let id = handle.id;
        self.agents.write().await.insert(id, handle);
        id
    }

    pub async fn list_agents(&self) -> Vec<Uuid> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    pub async fn send_message(&self, id: Uuid, msg: String) -> bool {
        let agents = self.agents.read().await;
        if let Some(agent) = agents.get(&id) {
            agent.tx.send(msg).await.is_ok()
        } else {
            false
        }
    }

    pub async fn stop_agent(&Self, id: Uuid) -> bool {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.remove(&id) {
            let _ = agent.shutdown.send(());
            true
        } else {
            false
        }
    }
}