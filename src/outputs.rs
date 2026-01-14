use std::sync::{Arc, Mutex};

pub trait OutputHandler: Send {
    fn display(&self, msg: String);
}

pub struct CliOutput;

impl OutputHandler for CliOutput {
    fn display(&self, msg: String) {
        println!("{}", msg);
    }
}

pub struct TuiOutput {
    messages: Arc<Mutex<Vec<String>>>,
}

impl TuiOutput {
    pub fn new(messages: Arc<Mutex<Vec<String>>>) -> Self {
        Self { messages }
    }
}

impl OutputHandler for TuiOutput {
    fn display(&self, msg: String) {
        if let Ok(mut msgs) = self.messages.lock() {
            msgs.push(msg);
        }
    }
}

pub type SharedOutput = Arc<dyn OutputHandler>;