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