use crate::prelude::*;
use strum::{EnumString, IntoStaticStr, EnumIter};
use std::str::FromStr;

#[derive(Debug)]
pub struct UserInput {
    os_info: OsInfo,
}

impl UserInput {

    pub fn new() -> Self {
        let os_info = OsInfo::new();
        UserInput{ os_info}
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
        let cmd = UserCommand::from_str(raw_input).unwrap_or(UserCommand::Unknown);

        match cmd {
            UserCommand::System => {
                let _output = self.os_info.display_all();
                InputAction::DoNothing
            },
            UserCommand::Quit | UserCommand::Exit => InputAction::Quit,

            UserCommand::Unknown => InputAction::ContinueNoSend(raw_input.to_string()),
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

    #[strum(disabled)]
    Unknown,
}
