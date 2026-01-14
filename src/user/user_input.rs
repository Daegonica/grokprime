use crate::prelude::*;
use strum::{EnumString, IntoStaticStr, EnumIter};
use std::str::FromStr;

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
        let cmd = UserCommand::from_str(raw_input).unwrap_or(UserCommand::Unknown);

        match cmd {
            UserCommand::System => {
                let output_text = self.os_info.display_all();
                self.output.display(output_text);
                InputAction::DoNothing
            },
            UserCommand::Quit | UserCommand::Exit => InputAction::Quit,

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

    #[strum(disabled)]
    Unknown,
}
