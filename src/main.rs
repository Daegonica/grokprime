use grokprime_brain::prelude::*;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::sync::Arc;
use ratatui::prelude::*;
use std::io::{stdout, Write};

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

    let output_buffer = app.get_message_buffer();
    let output: SharedOutput = Arc::new(TuiOutput::new(output_buffer));

    let user_input = UserInput::new(Arc::clone(&output));
    let mut shadow = GrokConnection::new(Arc::clone(&output));

    app.user_input = Some(user_input);
    app.add_message("Welcome to Shadow (TUI Mode)");
    app.add_message("Press ESC to exit");

    loop {
        app.flush_pending_messages();

        terminal.draw(|f| app.draw(f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Esc {
                    break;
                }

                if key.code == KeyCode::Enter && !app.input.trim().is_empty() {
                    let line = app.input.trim().to_string();

                    if let Some(ref user_input) = app.user_input {
                        match user_input.process_input(&line) {
                            InputAction::Quit => break,
                            InputAction::SendAsMessage(content) => {
                                app.add_message(format!("> {}", line));
                                shadow.add_user_message(&content);
                                if let Err(e) = shadow.handle_response().await {
                                    app.add_message(format!("Error: {}", e));
                                }
                            }
                            InputAction::DoNothing => {
                                app.add_message(format!("> {}", line));
                            }
                            InputAction::ContinueNoSend(msg) => {
                                app.add_message(format!("> {}", msg));
                                app.add_message(msg);
                            }
                        }
                    }
                    app.input.clear();
                } else {
                    app.handle_key(key);
                }
            }
        }
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

async fn run_cli_mode() -> Result<(), Box<dyn std::error::Error>> {
    let output: SharedOutput = Arc::new(CliOutput);

    let mut user_input = UserInput::new(Arc::clone(&output));
    let mut shadow = GrokConnection::new(Arc::clone(&output));

    println!("Welcome to Shadow (CLI Mode)");
    println!("Type 'quit' or 'exit' to leave");
    println!();

    loop {
        print!("Input: ");
        io::stdout().flush()?;

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
                    InputAction::ContinueNoSend(msg) => {
                        println!("{}", msg);
                    }
                    InputAction::DoNothing => {
                        continue;
                    }
                }
            }
            None => continue,
        }
    }

    Ok(())
}