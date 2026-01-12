use grokprime_brain::prelude::*;
use grokprime_brain::GrokConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shadow = GrokConnection::new();

    println!("Shadow awaits your command... \n   (type 'quit' to exit)");

    loop {
        match shadow.read_user_line()? {
            Some(raw_input) => {
                match shadow.process_input(&raw_input) {
                    InputAction::Quit => {
                        println!("Shadow retreats into the darkness...");
                        break;
                    }

                    InputAction::ContinueNoSend(msg) => {
                        println!("{}", msg);
                        continue;
                    }

                    InputAction::SendAsMessage(content) => {
                        shadow.add_user_message(&content);
                        if let Err(e) = shadow.handle_response().await {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
            }

            None => continue,
        }
    }
    let _ = save_history("conversation_history.json", &shadow.local_history);

    Ok(())
}


// Simple comment test for push/pulling on git. (1)

// Second test with ssh token for push/pull easier. (2)