use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Debug, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

// Success response
#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

// Error response from the API
#[derive(Deserialize, Debug)]
struct ApiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ApiErrorResponse {
    error: ApiErrorDetail,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    let api_key = env::var("GROK_API_KEY")?;
    let client = Client::new();

    // Conversation memory
    let mut messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are Grok, a helpful and maximally truthful AI dedicated to helping formulate/respond to tweets.".to_string(),
        },
    ];

    println!("Type your message and press Enter. Type 'exit' or 'quit' to stop.");

    loop {
        print!("You: ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();
        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            println!("Exiting chat.");
            break;
        }
        if user_input.is_empty() {
            continue;
        }

        // Add user message to memory
        messages.push(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        let request = ChatRequest {
            model: "grok-4-fast".to_string(),
            messages: messages.clone(),
            temperature: 0.7,
        };

        let response = match client
            .post("https://api.x.ai/v1/chat/completions")
            .bearer_auth(&api_key)
            .json(&request)
            .send()
            .await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Network or request error: {}", e);
                continue;
            }
        };

        let status = response.status();
        let text = match response.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to read response body: {}", e);
                continue;
            }
        };

        if status.is_success() {
            match serde_json::from_str::<ChatResponse>(&text) {
                Ok(chat_response) => {
                    if let Some(choice) = chat_response.choices.first() {
                        let reply = choice.message.content.trim();
                        println!("Grok: {}", reply);
                        // Add assistant reply to memory
                        messages.push(Message {
                            role: "assistant".to_string(),
                            content: reply.to_string(),
                        });
                    } else {
                        println!("Grok did not return a response.");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse success response: {}", e);
                    eprintln!("Raw response: {}", text);
                }
            }
        } else {
            match serde_json::from_str::<ApiErrorResponse>(&text) {
                Ok(error_body) => {
                    eprintln!("API Error: {}", error_body.error.message);
                    if let Some(code) = error_body.error.code {
                        eprintln!("Code: {}", code);
                    }
                }
                Err(_) => {
                    eprintln!("Request failed with status: {}", status);
                    eprintln!("Raw response: {}", text);
                }
            }
        }
    }

    Ok(())
}