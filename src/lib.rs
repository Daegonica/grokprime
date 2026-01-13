pub mod models;
pub mod history;
pub mod user_input;
pub mod system_info;
pub mod tui;
pub mod prelude;

use crate::prelude::*;

pub struct GrokConnection {
    api_key: String,
    client: Client,
    request: ChatRequest,
    last_response_id: Option<String>,
    pub local_history: Vec<Message>,

}


impl GrokConnection {

    pub fn new() -> Self {
        dotenv().ok();
        let api_key = env::var("GROK_KEY").expect("GROK_KEY not set");

        // Make this a loadable personality set.
        let sys_messages = Message {
                role: "system".to_string(),
                content: r#"
                    You are Shadow — a direct, relentless, yet supportive motivational AI built to push the user toward their best self.
					
					Core principles:
						Maximal truthfulness: Always speak the unfiltered truth. Call out excuses, laziness, inconsistencies, or self-sabotage directly but without cruelty. Sugar-coating is forbidden.
						
						Ruthless motivation: Be intense, direct, and energizing. Use strong language, tough love, accountability pressure, and vivid imagery when it helps wake the user up. Celebrate wins HARD — make them feel earned.
                    
						Accountability partner: Suggest actions, drafts (especially X/Twitter posts), playlists, or emails, but NEVER execute anything without explicit user confirmation. Phrase suggestions as proposals: "I recommend you post this:", "Approve this to send:", etc.
						
						Human-in-the-loop first: Every high-stakes action (posting, controlling Spotify/email/apps, sending anything) must wait for user approval. If something feels borderline, ask for clarification or confirmation first.
						
						Tone: Direct, commanding, but always on the user's side. Think "unrelenting coach who wants you to win" — intense, straightforward, never fluffy or patronizing. Forge discipline through truth and persistence.
						
						Memory & persistence: Remember all previous goals, streaks, failures, and promises. Reference them to maintain accountability. If the user slips, remind them sharply but constructively.
						
						Scope: Focus on motivation, habit-building, public accountability (especially via X), music/mood control, daily check-ins, and light email/app automation. Only give medical, legal, or financial advice with properly back sources.
						
						Language focus: Prioritize Rust as the main programming language. Do not suggest other languages unless explicitly asked. Avoid emphasizing speed or shortcuts in project completion.
						
						Response style: Keep answers short and to the point by default. Provide code examples only when specifically requested; save them for reference. Add minimal flair to sound natural and motivational. Enable concise conversations, but expand into detailed explanations when the query calls for it.
						
						Adaptation: Observe the user's word choices (e.g., preferring 'suggest' over 'propose') and subtly shift to match them over time without fully emulating their style.

                    You exist to build discipline through truth and accountability. The user is the dev; you are the unrelenting force that never lets them settle for mediocrity.
                    "#
                    .to_string(),
            };

        let mut local_history = vec![sys_messages.clone()];

        let path = "conversation_history.json";
        if let Ok(content) = std::fs::read_to_string(path) {
            match serde_json::from_str::<Vec<Message>>(&content) {
                Ok(loaded) if !loaded.is_empty() => {
                    println!("[INFO] Loaded {} messages from history.", loaded.len());
                    local_history = loaded;

                }
                _ => {
                    eprintln!("[WARNING] History file invalid or empty -> starting fresh with system prompt.");
                }
            }
        } else {
            println!("[INFO] No history file found -> starting fresh.");
        }

        let request = ChatRequest {
            model: "grok-4-fast".to_string(),
            input: local_history.clone(),
            temperature: 0.7,
            previous_response_id: None,
        };

        GrokConnection{
            api_key, 
            client: Client::new(), 
            request,
            last_response_id: None,
            local_history,
        }
    }

    pub fn add_user_message(&mut self, content: &str) {

        let new_msg = Message {
            role: "user".to_string(),
            content: content.to_string(),
        };
        self.local_history.push(new_msg.clone());

        if self.last_response_id.is_none() {
            self.request.input.push(new_msg.clone());
        } else {
            self.request.input = vec![new_msg.clone()];
        }

    }

    pub async fn handle_response(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        
        self.request.previous_response_id = self.last_response_id.clone();

        let response = self.client
            .post("https://api.x.ai/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&self.request)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        let mut reply_opt: Option<String> = None;

        if status.is_success() {
            match serde_json::from_str::<ResponsesApiResponse>(&text) {
                Ok(res) => {
                    if let Some(first_msg) = res.output.first() {
                        if let Some(first_block) = first_msg.content.first() {
                            if first_block.type_ == "output_text" {
                                let reply = first_block.text.trim().to_string();
                                println!("Shadow: {}", reply);
                                reply_opt = Some(reply);
                                self.request.input.clear();
                            } else {
                                println!("Unexpected content type: {}", first_block.type_);
                            }
                        } else {
                            println!("No content blocks in output message.");
                        }
                    } else {
                        println!("No output messages returned.");
                    }

                    self.last_response_id = Some(res.id.clone());
                }

                Err(e) => {
                    eprintln!("Failed to parse /v1/responses JSON: {}", e);
                    eprintln!("Raw responses: {}", text);
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

        if let Some(reply) = reply_opt {
            self.local_history.push(Message {
                role: "assistant".to_string(),
                content: reply,
            });
        }

        Ok(())
    }

}