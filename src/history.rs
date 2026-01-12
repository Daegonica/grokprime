use crate::prelude::*;

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct History {
    messages: Vec<Message>,
}

pub fn load_or_create_history(path: &str) -> History {
    if Path::new(path).exists() {
        match read_to_string(path) {
            Ok(content) => match serde_json::from_str::<History>(&content) {
                Ok(data) => {
                    println!("Loaded {} messages from {}", data.messages.len(), path);
                    data
                }
                Err(e) => {
                    eprintln!("JSON parse error: {}. Starting fresh.", e);
                    History { messages: vec![] }
                }
            },
            Err(e) => {
                eprintln!("Failed to read file: {}. Starting fresh.", e);
                History { messages: vec![] }
            }
        }
    } else  {
        println!("No history file found. Creating new one.");
        History { messages: vec![] }
    }
}

pub fn save_history(path: &str, history: &Vec<Message>) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(history)?;
    write(path, json)?;
    println!("Saved history ({} messages)", history.len());
    Ok(())
}
use crate::prelude::*;

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct History {
    messages: Vec<Message>,
}

pub fn load_or_create_history(path: &str) -> History {
    if Path::new(path).exists() {
        match read_to_string(path) {
            Ok(content) => match serde_json::from_str::<History>(&content) {
                Ok(data) => {
                    println!("Loaded {} messages from {}", data.messages.len(), path);
                    data
                }
                Err(e) => {
                    eprintln!("JSON parse error: {}. Starting fresh.", e);
                    History { messages: vec![] }
                }
            },
            Err(e) => {
                eprintln!("Failed to read file: {}. Starting fresh.", e);
                History { messages: vec![] }
            }
        }
    } else  {
        println!("No history file found. Creating new one.");
        History { messages: vec![] }
    }
}

pub fn save_history(path: &str, history: &Vec<Message>) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(history)?;
    write(path, json)?;
    println!("Saved history ({} messages)", history.len());
    Ok(())
}