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

use grokprime_brain::prelude::*;
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
    let args = Args::parse();

    if args.is_tui_mode() {
        run_tui_mode().await?;
    } else {
        run_cli_mode().await?;
    }

    Ok(())
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
    log_init("Shadow", Some("shadow.log"), OutputTarget::LogFile)?;
    log_info!("Starting Shadow in TUI mode");

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    log_info!("Switching to alternate screen");

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // TUI setup
    log_info!("Setting up TUI application");
    let mut app = ShadowApp::new();
    let persona_paths: Vec<&Path> = vec![
        Path::new("personas/shadow/shadow.yaml"), 
        Path::new("personas/friday/friday.yaml")
        ];
    
    log_info!("Adding personas from paths: {:?}", persona_paths);
    app.load_personas(persona_paths).expect("Failed to load personas");
    if let Some(persona_ref) = app.personas.get("shadow") {
        let id = Uuid::new_v4();
        app.add_agent(id, Arc::clone(persona_ref));
        app.current_agent = Some(id);
    } else {
        eprintln!("Error: Shadow persona not found!");
        std::process::exit(1);
    }

    let user_input = UserInput::new_for_tui();

    app.user_input = Some(user_input);
    app.add_message("Welcome to Shadow (TUI Mode)");
    app.add_message("Press ESC to exit");

    log_info!("Entering main event loop");
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
    
    log_info!("Exiting main event loop");
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    log_info!("Reverted to main screen");

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
async fn run_cli_mode() -> Result<(), Box<dyn std::error::Error>> {
    let output: SharedOutput = Arc::new(CliOutput);

    let mut user_input = UserInput::new(Some(Arc::clone(&output)));
    let persona = Persona::from_yaml_file(Path::new("personas/shadow.yaml"))
        .expect("Failed to load shadow persona");
    let mut shadow = GrokConnection::new(output.clone(), Arc::new(persona));
    let twitter = TwitterConnection::new(Arc::clone(&output));

    println!("Welcome to Shadow (CLI Mode)");
    println!("Type 'quit' or 'exit' to leave");
    println!();

    loop {

        match user_input.read_user_input()? {
            Some(raw_input) => {
                match user_input.process_input(&raw_input) {
                    InputAction::Quit => {
                        println!("Shadow retreats into the darkness...");
                        break;
                    }
                    InputAction::SendAsMessage(content) => {
                        shadow.add_user_message(&content);
                        if let Err(e) = shadow.handle_response().await {
                            eprintln!("Error: {}", e);
                        }
                    }
                    InputAction::PostTweet(tweet_text) => {
                        match twitter.post_tweet(&tweet_text).await {
                            Ok(_) => println!("✓ Tweet posted!"),
                            Err(e) => eprintln!("✗ Failed: {}", e),
                        }
                    }
                    InputAction::DraftTweet(idea) => {
                        let prompt = format!(
                            "Generate a tweet based on this idea: '{}'. \
                            Keep it under 280 characters. Return ONLY the tweet text, \
                            speak as me, but sprinkle in some fourth wall breaking of your own choosing.",
                            idea
                        );
                        
                        shadow.add_user_message(&prompt);
                        if let Err(e) = shadow.handle_response().await {
                            eprintln!("Error: {}", e);
                        } else {
                            println!("\nTo post, type: tweet <approved text>");
                        }
                    }
                    InputAction::ContinueNoSend(msg) => {
                        println!("{}", msg);
                    }
                    InputAction::DoNothing => {
                        continue;
                    }

                    
                   InputAction::HistoryInfo | InputAction::SaveHistory | InputAction::Summarize | InputAction::ClearHistory | InputAction::NewAgent(_) | InputAction::CloseAgent | InputAction::ListAgents | InputAction::AgentStatus => todo!(),
                }
            }
            None => continue,
        }
    }
    
    let _ = shadow.save_history("conversation_history.json");

    Ok(())
}