use clap::ColorChoice;
use clap::Parser;

use crate::commands::formatting::FormatArgs;
use crate::commands::formatting::FormatTool;
use crate::commands::outcome::CommandOutcome;
use crate::config::Configuration;
use crate::error::Error;

/// Format PHP source files.
#[derive(Parser, Debug, Default)]
#[command(name = "format", aliases = ["fmt"], about = "Format PHP source files.")]
pub struct FormatCommand {
    #[command(flatten)]
    pub(crate) arguments: FormatArgs,
}

impl FormatCommand {
    pub fn execute(self, configuration: Configuration, color: ColorChoice) -> Result<CommandOutcome, Error> {
        self.arguments.execute(FormatTool::Php, configuration, color)
    }
}
