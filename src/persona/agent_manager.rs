// This is the 'gate' into the agent_registry and there for the agents themselves
// This is where we do everything from calling methods to start/stop
    // Display/Store messages

// Essentially any methods made in agent_reg or agent.rs is called here for the CLI/TUI modes to call on.
use std::collections::HashMap;
use uuid::Uuid;

use crate::prelude::*;
use crate::persona::agent::AgentInfo;


#[derive(Debug)]
pub struct AgentManager {
    pub personas: HashMap<String, PersonaRef>,
    pub agents: HashMap<Uuid, AgentInfo>,
    pub current_agent: Option<Uuid>,
    pub agent_order: Vec<Uuid>,
    pub user_input: Option<UserInput>,
}

impl AgentManager {

    pub fn new() -> Self {
        Self {
            personas: HashMap::new(),
            agents: HashMap::new(),
            current_agent: None,
            agent_order: Vec::new(),
            user_input: None,
        }
    }

    pub fn load_personas(&mut self, personas_paths: Vec<&Path>) -> anyhow::Result<()> {
        for path in personas_paths {
            let persona = Persona::from_yaml_file(path)?;// Quickly deal with errors
            self.personas.insert(persona.name.clone(), Arc::new(persona));
        }

        Ok(())
    }

    pub fn add_agent(&mut self, id: Uuid, persona: PersonaRef) {

        let agent = AgentInfo::new(id, persona);
        self.agent_order.push(id);
        self.current_agent = Some(id);
        self.agents.insert(id, agent);

    }

    pub fn remove_agent(&mut self, id: Uuid) {
        if let Some(agent) = self.agents.get_mut(&id) {
            if let Some(task) = agent.active_task.take() {
                task.abort();
            }
        }

        self.agents.remove(&id);
        self.agent_order.retain(|&x| x != id);
        if self.current_agent == Some(id) {
            self.current_agent = self.agent_order.last().cloned();
        }

    }

    pub fn get_agent_name(&self, id: Uuid) -> String {
        self.agents.get(&id)
            .map(|agent| agent.persona_name.clone())
            .unwrap_or("<unknown>".to_string())
    }

    pub fn switch_agent(&mut self, next: bool) {
        if self.agent_order.is_empty() {return;}

        if let Some(current) = self.current_agent {
            let idx = self.agent_order.iter().position(|&x| x == current).unwrap_or(0);
            let new_idx = if next {
                (idx + 1) % self.agent_order.len()
            } else {
                (idx + self.agent_order.len() - 1) % self.agent_order.len()
            };
            self.current_agent = Some(self.agent_order[new_idx]);
        } else {
            self.current_agent = self.agent_order.first().cloned();
        }
    }

    pub fn current_pane(&self) -> Option<&AgentInfo> {
        self.current_agent.and_then(|id| self.agents.get(&id))
    }

    pub fn current_pane_mut(&mut self) -> Option<&mut AgentInfo> {
        self.current_agent.and_then(move |id| self.agents.get_mut(&id))
    }

    pub fn poll_channels(&mut self) {
        for (_, agent) in self.agents.iter_mut() {

            while let Ok(chunk) = agent.chunk_receiver.try_recv() {
                match chunk {
                    StreamChunk::Delta(text) => {
                        if let Some(last_msg) = agent.messages.back_mut() {
                            if !last_msg.starts_with('>') {
                                last_msg.push_str(&text);
                            } else {
                                agent.add_message(text);
                            }
                        } else {
                            agent.add_message(text);
                        }
                    }

                    StreamChunk::Complete{response_id, full_reply} => {
                        agent.connection.set_last_response_id(response_id.clone());

                        agent.connection.conversation.local_history.push(Message {
                            role: "assistant".to_string(),
                            content: full_reply,
                        });

                        agent.is_waiting = false;
                        agent.active_task = None;
                    }

                    StreamChunk::Error(err) => {
                        agent.add_message(format!("Error: {}", err));
                        agent.add_message("Type you message again to retry.");
                        agent.is_waiting = false;
                        agent.active_task = None;
                    }

                    StreamChunk::Info(msg) => {
                        log_info!("Info: {}", msg);
                    }
                }
            }
        }
    }

}