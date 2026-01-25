//! # Daegonica Module: main
//!
//! **Purpose:** Entry point for the GrokPrime-Brain application
//!
//! **Context:**
//! - This is the main executable entry point that initializes and runs the application
//! - Supports both TUI (Terminal User Interface) and CLI (Command Line Interface) modes
//!
//! **Responsibilities:**
//! - Parse command-line arguments to determine run mode
//! - Initialize and manage the application lifecycle
//! - Set up terminal environments for TUI mode
//! - Coordinate user input, Grok API connections, and output handlers
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-18
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use grokprime_brain::persona::discover_personas;
use grokprime_brain::{
    prelude::*,
    commands::{from_input_action, CommandResult},
};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use uuid::Uuid;
use std::sync::Arc;
use ratatui::prelude::*;
use std::io::stdout;
use std::time::Duration;



/// # main
///
/// **Purpose:**
/// Application entry point that determines and executes the appropriate run mode.
///
/// **Parameters:**
/// None (arguments parsed internally via clap)
///
/// **Returns:**
/// `Result<(), Box<dyn std::error::Error>>` - Success or propagated error
///
/// **Errors / Failures:**
/// - Terminal initialization failures in TUI mode
/// - API connection errors
/// - File I/O errors when saving history
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    log_init("Shadow", Some("logs/shadow.log"), OutputTarget::LogFile)?;

    let args = Args::parse();

    if args.is_tui_mode() {
        run_tui_mode().await?;
    } else {
        run_cli_mode(&args.persona).await?;
    }

    Ok(())
}

enum CurrentMode {
    Shadow(ShadowApp),
    Manager(AgentManager),
}

fn initialize_app(
    default_persona: &str,
    for_cli: bool,
) -> anyhow::Result<CurrentMode> {

    let personas = discover_personas()?;
    let persona_paths: Vec<&Path> = personas.iter()
        .map(|(_, path_buf)| path_buf.as_path())
        .collect();

    log_info!("Loading personas from paths: {:?}", persona_paths);

    let user_input = if for_cli {
        UserInput::new(Some(Arc::new(CliOutput)))
    } else {
        UserInput::new_for_tui()
    };

    if for_cli {

        let mut agent_manager = AgentManager::new();
        agent_manager.load_personas(persona_paths.clone())?;
        agent_manager.user_input = Some(user_input);

        log_info!("Starting Shadow in CLI mode");
        println!("Welcome to Shadow (CLI Mode)");
        println!("Type 'quit' or 'exit' to leave");

        agent_manager.load_personas(persona_paths)?;
    
        if let Some(persona_ref) = agent_manager.personas.get(default_persona) {
            let id = Uuid::new_v4();
            agent_manager.add_agent(id, Arc::clone(persona_ref));
            agent_manager.current_agent = Some(id);
            log_info!("Added default agent: {}", default_persona);
        } else {
            anyhow::bail!("Persona '{}' not found!", default_persona);
        }

        Ok(CurrentMode::Manager(agent_manager))
    } else {

        let mut app = ShadowApp::new();
        app.load_personas(persona_paths)?;
        app.user_input = Some(user_input);

        log_info!("Starting Shadow in TUI mode");
        app.add_message("Welcome to Shadow (TUI Mode)");
        app.add_message("Press ESC to exit");
    
        if let Some(persona_ref) = app.personas.get(default_persona) {
            let id = Uuid::new_v4();
            app.add_agent(id, Arc::clone(persona_ref));
            app.current_agent = Some(id);
            log_info!("Added default agent: {}", default_persona);
        } else {
            anyhow::bail!("Persona '{}' not found!", default_persona);
        }

        Ok(CurrentMode::Shadow(app))
    }
}


/// # run_tui_mode
///
/// **Purpose:**
/// Initializes and runs the application in TUI (Terminal User Interface) mode with full
/// interactive display, message history, and real-time updates.
///
/// **Parameters:**
/// None
///
/// **Returns:**
/// `Result<(), Box<dyn std::error::Error>>` - Success or propagated error
///
/// **Errors / Failures:**
/// - Terminal raw mode enabling failures
/// - Screen buffer initialization errors
/// - Event handling errors during the main loop
/// - History save failures on exit
///
/// **Examples:**
/// ```rust
/// // Called automatically when --tui flag is set (default)
/// run_tui_mode().await?;
/// ```
async fn run_tui_mode() -> Result<(), Box<dyn std::error::Error>> {

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let CurrentMode::Shadow(mut app) = initialize_app("shadow", false)? else {
        panic!("Expected Shadow variant in TUI mode.");
    };

    loop {
        app.poll_channels();
        terminal.draw(|f| app.draw(f))?;

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let should_continue = app.handle_key(key);
                    if !should_continue {
                        break;
                    }
                }
            }
        }
    }
    
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// # run_cli_mode
///
/// **Purpose:**
/// Runs the application in CLI (Command Line Interface) mode with simple text-based
/// input/output for scripting and automation scenarios.
///
/// **Parameters:**
/// None
///
/// **Returns:**
/// `Result<(), Box<dyn std::error::Error>>` - Success or propagated error
///
/// **Errors / Failures:**
/// - Standard input reading failures
/// - API communication errors
/// - Twitter posting errors
/// - History save failures on exit
///
/// **Examples:**
/// ```rust
/// // Called when --cli flag is specified
/// run_cli_mode().await?;
/// ```
async fn run_cli_mode(persona: &str) -> Result<(), Box<dyn std::error::Error>> {

    let CurrentMode::Manager(mut app) = initialize_app(persona, true)? else {
        panic!("Expected Manager variant in CLI mode.");
    };

    loop {

        let user_input = app.user_input.as_mut().unwrap();

        match user_input.read_user_input()? {
            Some(raw_input) => {
                match user_input.process_input(&raw_input) {
                    InputAction::DoNothing => {},
                    InputAction::ContinueNoSend(msg) => {
                        println!("{}", msg);
                    }

                    InputAction::SendAsMessage(content) => {
                        if let Some(agent) = app.current_pane_mut() {
                            agent.add_message(format!("> {}", content));
                            agent.connection.add_user_message(&content);
                            
                            let msg_count_before = agent.messages.len();

                            println!("Shadow is thinking...\n");
                            
                            if let Err(e) = agent.connection.handle_response().await {
                                eprintln!("Error: {}", e);
                                continue;
                            }

                            loop {
                                tokio::time::sleep(Duration::from_millis(50)).await;

                                app.poll_channels();

                                if let Some(agent) = app.current_pane() {
                                    if agent.messages.len() > msg_count_before {
                                        if let Some(last_msg) = agent.messages.back() {
                                            if !last_msg.starts_with('>') {
                                                print!("\r{}", last_msg);
                                                std::io::stdout().flush().unwrap();
                                            }
                                        }
                                    }

                                    if !agent.is_waiting {
                                        println!("\n");
                                        break;
                                    }
                                }
                            }
                        } else {
                            println!("No active agent!");
                        }
                    }


                    action => {
                        let command = from_input_action(action);
                        let result = command.execute(&mut app);

                        match result {
                            CommandResult::Continue => {},
                            CommandResult::Shutdown => {
                                println!("Shadow retreats into the darkness...");
                                break;
                            }
                            CommandResult::Error(msg) => {
                                eprintln!("Error: {}", msg);
                            }
                        }
                    }
                }
            }
            None => continue,
        }
    }
    
    if let Some(agent) = app.current_pane_mut() {
        let _ = agent.connection.save_persona_history();
    }

    Ok(())
}