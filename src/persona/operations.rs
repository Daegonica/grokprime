use crate::persona::agent::AgentInfo;
use crate::persona::agent_manager::AgentManager;
use uuid::Uuid;
use crate::prelude::*;

pub trait AgentOperations {

    fn current_agent_info(&self) -> Option<&AgentInfo>;
    fn current_agent_info_mut(&mut self) -> Option<&mut AgentInfo>;

    fn get_agent_info(&self, id: Uuid) -> Option<&AgentInfo>;
    fn get_agent_info_mut(&mut self, id: Uuid) -> Option<&mut AgentInfo>;

    fn display_message(&mut self, msg: String);

    fn add_new_agent(&mut self, id: Uuid, persona: PersonaRef);
    fn remove_agent(&mut self, id: Uuid);

    fn get_persona(&self, name: &str) -> Option<PersonaRef>;
    fn get_current_agent_id(&self) -> Option<Uuid>;
    fn set_current_agent_id(&mut self, id: Option<Uuid>);
    fn get_agent_order(&self) -> &Vec<Uuid>;
    fn get_all_agent_names(&self) -> Vec<(Uuid, String)>;
}

impl AgentOperations for AgentManager {
    fn current_agent_info(&self) -> Option<&AgentInfo> {
        self.current_pane()
    }

    fn current_agent_info_mut(&mut self) -> Option<&mut AgentInfo> {
        self.current_pane_mut()
    }

    fn get_agent_info(&self, id: Uuid) -> Option<&AgentInfo> {
        self.agents.get(&id)
    }

    fn get_agent_info_mut(&mut self, id: Uuid) -> Option<&mut AgentInfo> {
        self.agents.get_mut(&id)
    }

    fn display_message(&mut self, msg: String) {
        println!("{}", msg);
    }

    fn add_new_agent(&mut self, id: Uuid, persona: PersonaRef) {
        self.add_agent(id, persona);
    }

    fn remove_agent(&mut self, id: Uuid) {
        self.remove_agent(id);
    }
    
    fn get_persona(&self, name: &str) -> Option<PersonaRef> {
        self.personas.get(name).cloned()
    }
    
    fn get_current_agent_id(&self) -> Option<Uuid> {
        self.current_agent
    }
    
    fn set_current_agent_id(&mut self, id: Option<Uuid>) {
        self.current_agent = id;
    }
    
    fn get_agent_order(&self) -> &Vec<Uuid> {
        &self.agent_order
    }
    
    fn get_all_agent_names(&self) -> Vec<(Uuid, String)> {
        self.agents.iter()
            .map(|(id, agent)| (*id, agent.persona_name.clone()))
            .collect()
    }
}

impl AgentOperations for ShadowApp {
    fn current_agent_info(&self) -> Option<&AgentInfo> {
        self.agent_manager.current_pane()
    }

    fn current_agent_info_mut(&mut self) -> Option<&mut AgentInfo> {
        self.agent_manager.current_pane_mut()
    }

    fn get_agent_info(&self, id: Uuid) -> Option<&AgentInfo> {
        self.agent_manager.agents.get(&id)
    }

    fn get_agent_info_mut(&mut self, id: Uuid) -> Option<&mut AgentInfo> {
        self.agent_manager.agents.get_mut(&id)
    }

    fn display_message(&mut self, msg: String) {
        self.add_message(msg);
    }

    fn add_new_agent(&mut self, id: Uuid, persona: PersonaRef) {
        self.add_agent(id, persona);
    }

    fn remove_agent(&mut self, id: Uuid) {
        self.remove_agent(id);
    }
    
    fn get_persona(&self, name: &str) -> Option<PersonaRef> {
        self.agent_manager.personas.get(name).cloned()
    }
    
    fn get_current_agent_id(&self) -> Option<Uuid> {
        self.agent_manager.current_agent
    }
    
    fn set_current_agent_id(&mut self, id: Option<Uuid>) {
        self.agent_manager.current_agent = id;
    }
    
    fn get_agent_order(&self) -> &Vec<Uuid> {
        &self.agent_manager.agent_order
    }
    
    fn get_all_agent_names(&self) -> Vec<(Uuid, String)> {
        self.agent_manager.agents.iter()
            .map(|(id, agent)| (*id, agent.persona_name.clone()))
            .collect()
    }
}