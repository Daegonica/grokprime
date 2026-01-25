// Store all information related to an Agent that can be used in CLI/TUI modes
use uuid::Uuid;
use std::collections::VecDeque;

use crate::prelude::*;

use crate::llm::{
    client::Connection,
    AnyClient,
};
use crate::grok::client::GrokClient;
use crate::claude::client::ClaudeClient;

type DynamicConnection = Connection<AnyClient>;

#[derive(Debug)]
pub struct AgentInfo {

    pub id: Uuid,
    pub persona_name: String,
    pub connection: DynamicConnection,
    pub messages: VecDeque<String>,
    pub is_waiting: bool,

    pub chunk_receiver: mpsc::UnboundedReceiver<StreamChunk>,
    pub chunk_sender: mpsc::UnboundedSender<StreamChunk>,

    pub active_task: Option<tokio::task::JoinHandle<()>>,

}

impl AgentInfo {

    pub fn new(id: Uuid, persona: PersonaRef) -> Self {

        let client = match persona.api_provider.as_str() {
            "claude" => AnyClient::Claude(ClaudeClient::new().expect("Failed to init Claude.")),
            _ => AnyClient::Grok(GrokClient::new().expect("Failed to init Grok.")),
        };
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            id,
            persona_name: persona.name.clone(),
            connection: Connection::new_without_output(client, persona),
            messages: VecDeque::new(),
            is_waiting: false,

            chunk_receiver: rx,
            chunk_sender: tx,

            active_task: None,
        }
    }

    pub fn add_message(&mut self, msg: impl Into<String>) {
        self.messages.push_back(msg.into());
    }

}