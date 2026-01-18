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
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = ShadowApp::new();
    let default_persona = "shadow".to_string();
    let default_id = Uuid::new_v4();
    app.add_agent(default_id, default_persona);
    app.current_agent = Some(default_id);

    let output_buffer = app.get_message_buffer();
    let output: SharedOutput = Arc::new(TuiOutput::new(output_buffer));

    let user_input = UserInput::new(Arc::clone(&output));
    let shadow = GrokConnection::new(Arc::clone(&output));
    // let twitter = TwitterConnection::new(Arc::clone(&output));

    app.user_input = Some(user_input);
    app.add_message("Welcome to Shadow (TUI Mode)");
    app.add_message("Press ESC to exit");

    loop {
        app.flush_pending_messages();
        terminal.draw(|f| app.draw(f))?;
    
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let should_continue = app.handle_key(key);
                if !should_continue {
                    break;
                }
                app.flush_pending_messages();
                terminal.draw(|f| app.draw(f))?;
            }
        }
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    
    let _ = shadow.save_history("conversation_history.json");

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

    let mut user_input = UserInput::new(Arc::clone(&output));
    let mut shadow = GrokConnection::new(Arc::clone(&output));
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
                    InputAction::NewAgent(_) | InputAction::CloseAgent | InputAction::ListAgents => todo!(),
                }
            }
            None => continue,
        }
    }
    
    let _ = shadow.save_history("conversation_history.json");

    Ok(())
}