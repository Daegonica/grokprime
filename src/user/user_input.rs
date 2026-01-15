use crate::prelude::*;
use strum::{EnumString, IntoStaticStr, EnumIter};
use std::str::FromStr;

#[derive(Clone)]
pub struct UserInput {
    os_info: OsInfo,
    output: SharedOutput,
}

impl std::fmt::Debug for UserInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserInput")
            .field("os_info", &self.os_info)
            .field("output", &"<OutputHandler>")
            .finish()
    }
}

impl UserInput {

    pub fn new(output: SharedOutput) -> Self {
        let os_info = OsInfo::new();
        UserInput{ os_info, output}
    }

    pub fn read_user_input(&mut self) -> std::io::Result<Option<String>> {
        print!("Input: ");
        io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    }

    pub fn process_input(&self, raw_input: &str) -> InputAction {
        let parts: Vec<&str> = raw_input.splitn(2, ' ').collect();
        let potential_command = parts[0];
        let remainder = if parts.len() > 1 { parts[1] } else { "" };

        let cmd = UserCommand::from_str(potential_command).unwrap_or(UserCommand::Unknown);

        match cmd {
            UserCommand::System => {
                let output_text = self.os_info.display_all();
                InputAction::ContinueNoSend(output_text)
            },
            UserCommand::Quit | UserCommand::Exit => InputAction::Quit,

            UserCommand::Tweet => {
                if remainder.is_empty() {
                    self.output.display("Usage: tweet <your message>".to_string());
                    InputAction::DoNothing
                } else {
                    InputAction::PostTweet(remainder.to_string())
                }
            },

            UserCommand::Draft => {
                if remainder.is_empty() {
                    self.output.display("Usage: draft <your idea>".to_string());
                    InputAction::DoNothing
                } else {
                    InputAction::DraftTweet(remainder.to_string())
                }
            },
            UserCommand::New => {
                if remainder.is_empty() {
                    self.output.display("Usage: new <persona>".to_string());
                    InputAction::DoNothing
                } else {
                    InputAction::NewAgent(remainder.to_string())
                }
            },
            UserCommand::Close => InputAction::CloseAgent,
            UserCommand::List => InputAction::ListAgents,

            UserCommand::Unknown => InputAction::SendAsMessage(raw_input.to_string()),
        }
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr, EnumIter)]
#[strum(serialize_all = "lowercase")]
#[strum(ascii_case_insensitive)]
enum UserCommand {
    System,
    Quit,
    Exit,
    Tweet,
    Draft,
    New,
    Close,
    List,

    #[strum(disabled)]
    Unknown,
}
