//! # Daegonica Module: tui::command_handler
//!
//! **Purpose:** Command execution handlers for TUI user input
//!
//! **Context:**
//! - Processes InputAction commands from user input
//! - Executes operations on agents and application state
//! - Spawns async tasks for API communication
//!
//! **Responsibilities:**
//! - Handle message sending to Grok API
//! - Manage history operations (save, clear, summarize)
//! - Control agent lifecycle (new, close, status)
//! - Route commands to appropriate handlers
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use uuid::Uuid;
use crate::prelude::*;
use crate::tui::app::ShadowApp;

/// # handle_agent_status
///
/// **Purpose:**
/// Displays status information about all agents and the current selection.
///
/// **Parameters:**
/// - `app`: Mutable reference to the TUI application state
///
/// **Returns:**
/// None (adds status message to app)
pub fn handle_agent_status(app: &mut ShadowApp) {
    let mut status = String::new();
    status.push_str(&format!("Current agent: {}\n", app.current_agent
        .and_then(|id| app.agents.get(&id))
        .map(|pane| capitalize_first(&pane.persona_name))
        .unwrap_or("<none>".to_string())));

    status.push_str(&format!(" - Current pane: {}\n", app.current_pane_mut()
        .map(|pane| capitalize_first(&pane.persona_name))
        .unwrap_or("<none>".to_string())));

    status.push_str(" - All agents:\n");

    for id in &app.agent_order {
        let pane = &app.agents[id];
        let marker = if Some(*id) == app.current_agent {" ->"} else {" "};
        status.push_str(&format!("{} {}\n", marker, capitalize_first(&pane.persona_name)));
    }

    status.push_str(&format!(" - Total tabs: {}", app.agent_order.len()));

    app.add_message(format!("{}", status));
}

/// # handle_save_history
///
/// **Purpose:**
/// Saves the current agent's conversation history to disk.
///
/// **Parameters:**
/// - `app`: Mutable reference to the TUI application state
///
/// **Returns:**
/// None (displays success/error message)
pub fn handle_save_history(app: &mut ShadowApp) {
    if let Some(pane) = app.current_pane_mut() {
        let result = pane.connection.save_persona_history();
        let persona_name = pane.connection.persona.name.clone();
        match result {
            Ok(_) => {
                app.add_message(format!("History saved for {}", persona_name));
                log_info!("History saved for {}", persona_name);
            },
            Err(e) => {
                log_error!("Failed to save history: {}", e);
                app.add_message(format!("Failed to save history: {}", e));
            }
        }
    } else {
        app.add_message("No agent available to save history for.");
    }
}

/// # handle_history_info
///
/// **Purpose:**
/// Displays information about the current agent's conversation history.
///
/// **Parameters:**
/// - `app`: Mutable reference to the TUI application state
///
/// **Returns:**
/// None (displays history stats)
pub fn handle_history_info(app: &mut ShadowApp) {
    if let Some(pane) = app.current_pane_mut() {
        let msg_count = pane.connection.local_history.len();
        let has_summary = pane.connection.local_history.iter()
            .any(|m| m.content.contains("[Previous conversation summary:"));
        let persona_name = pane.connection.persona.name.clone();
        log_info!("{}: {} messages, Summary present: {}", persona_name, msg_count, has_summary);
        app.add_message(format!(
            "History for {}: {} messages, Summary present: {}",
            persona_name, msg_count, has_summary
        ));
    } else {
        app.add_message("No agent available.");
    }
}

/// # handle_clear_history
///
/// **Purpose:**
/// Clears the history file for the current agent from disk.
///
/// **Parameters:**
/// - `app`: Mutable reference to the TUI application state
///
/// **Returns:**
/// None (displays success/error message)
pub fn handle_clear_history(app: &mut ShadowApp) {
    if let Some(pane) = app.current_pane_mut() {
        let persona_name = pane.connection.persona.name.clone();
        let path = format!("history/{}_history.json", &persona_name);
        let result = std::fs::remove_file(&path);
        match result {
            Ok(_) => {
                log_info!("Cleared history for {}", persona_name);
                app.add_message(format!("Cleared history for {}", persona_name));
            },
            Err(_) => {
                log_error!("No history file found for {}", persona_name);
                app.add_message(format!("No history file found for {}", persona_name));
            }
        }
    } else {
        app.add_message("No agent available.");
    }
}

/// # handle_new_agent
///
/// **Purpose:**
/// Creates a new agent with the specified persona.
///
/// **Parameters:**
/// - `app`: Mutable reference to the TUI application state
/// - `persona_name`: Name of the persona to load
///
/// **Returns:**
/// None (creates agent or displays error)
pub fn handle_new_agent(app: &mut ShadowApp, persona_name: String) {
    if let Some(persona_ref) = app.personas.get(&persona_name) {
        let id = Uuid::new_v4();
        app.add_agent(id, Arc::clone(persona_ref));
        app.current_agent = Some(id);
        app.add_message(format!(
            "Created new agent with persona '{}'",
            capitalize_first(&persona_name)
        ));
    } else {
        app.add_message(format!(
            "Persona '{}' not found.",
            capitalize_first(&persona_name)
        ));
    }
}

/// # handle_close_agent
///
/// **Purpose:**
/// Closes the current agent and removes it from the application.
///
/// **Parameters:**
/// - `app`: Mutable reference to the TUI application state
///
/// **Returns:**
/// None (removes agent or displays error)
pub fn handle_close_agent(app: &mut ShadowApp) {
    if let Some(id) = app.current_agent {
        app.remove_agent(id);
        app.add_message("Closed current agent.");
    } else {
        app.add_message("No agent to close.");
    }
}

/// # handle_send_message
///
/// **Purpose:**
/// Sends a message to the Grok API via the current agent.
///
/// **Parameters:**
/// - `app`: Mutable reference to the TUI application state
/// - `content`: The message content to send
///
/// **Returns:**
/// None (spawns async task to handle response)
pub fn handle_send_message(app: &mut ShadowApp, content: String) {
    if let Some(pane) = app.current_pane_mut() {
        pane.add_message(format!("> {}", content));
        pane.is_waiting = true;

        if let Some(old_task) = pane.active_task.take() {
            old_task.abort();
        }

        let mut connection = pane.connection.clone();
        let tx = pane.chunk_sender.clone();
        let content_owned = content.to_string();

        let handle = tokio::spawn(async move {
            connection.add_user_message(&content_owned);
            if let Err(e) = connection.handle_response_streaming(tx.clone()).await {
                let _ = tx.send(StreamChunk::Error(format!("{}",e)));
            }
        });

        pane.active_task = Some(handle);
    } else {
        app.add_message("No agent available. Create one with 'new <persona>'");
    }
}

/// # handle_summarize
///
/// **Purpose:**
/// Triggers conversation history summarization for the current agent.
///
/// **Parameters:**
/// - `app`: Mutable reference to the TUI application state
///
/// **Returns:**
/// None (spawns async task to summarize)
pub fn handle_summarize(app: &mut ShadowApp) {
    if let Some(pane) = app.current_pane_mut() {
        let mut conn = pane.connection.clone();
        let tx = pane.chunk_sender.clone();
        app.add_message("Summarization started...");
        tokio::spawn(async move {
            tx.send(StreamChunk::Info("Starting summarization...".to_string())).ok();
            if let Err(e) = conn.summarize_history().await {
                tx.send(StreamChunk::Error(format!("Summarization error: {}", e))).ok();
            } else {
                tx.send(StreamChunk::Info("Summarization complete.".to_string())).ok();
                if let Err(e) = conn.save_persona_history() {
                    tx.send(StreamChunk::Error(format!("Failed to save persona history: {}", e))).ok();
                }
            }
        });
        app.add_message("Summarization finished.");
    } else {
        app.add_message("No agent available.");
    }
}