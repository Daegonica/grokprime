use crate::persona::PersonaRef;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub struct AgentHandle {
    pub id: Uuid,
    pub tx: mpsc:: Sender<String>,
    pub shutdown: oneshit::Sender<()>,
}

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