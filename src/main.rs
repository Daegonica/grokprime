use grokprime_brain::prelude::*;
use grokprime_brain::GrokConnection;
use crossterm::{
    event::{self, Event, KeyEventKind, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::stdout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = ShadowApp::new();

    let mut shadow = GrokConnection::new();

    app.add_message("Welcome");

    loop {
        terminal.draw(|f| app.draw(f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Esc {
                    break;
                }
                app.handle_key(key);
            }
        }
        // match user.read_user_input()? {
        //     Some(raw_input) => {
        //         match user.process_input(&raw_input) {
        //             InputAction::Quit => {
        //                 println!("Shadow retreats into the darkness...");
        //                 break;
        //             }

        //             InputAction::SendAsMessage(content) => todo!(),

        //             InputAction::ContinueNoSend(msg) => {
        //                 println!("{}", msg);
        //                 continue;
        //             }

        //             InputAction::DoNothing => {continue;}
        //         }
        //     }

        //     None => continue,
        // }


        // match shadow.read_user_line()? {
        //     Some(raw_input) => {
        //         match shadow.process_input(&raw_input) {

        //             InputAction::SendAsMessage(content) => {
        //                 shadow.add_user_message(&content);
        //                 if let Err(e) = shadow.handle_response().await {
        //                     eprintln!("Error: {}", e);
        //                 }
        //             }
        //         }
        //     }

        //     None => continue,
        // }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    // let _ = save_history("conversation_history.json", &shadow.local_history);

    Ok(())
}


// Simple comment test for push/pulling on git. (1)

// Second test with ssh token for push/pull easier. (2)