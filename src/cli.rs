use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "grokprime-brain")]
#[command(about = "Shadow AI Assistant", long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = true)]
    pub tui: bool,

    #[arg(long, conflicts_with = "tui")]
    pub cli: bool,
}

impl Args {
    pub fn is_tui_mode(&self) -> bool {
        !self.cli
    }
}