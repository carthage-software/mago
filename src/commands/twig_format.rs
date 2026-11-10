use clap::ColorChoice;
use clap::Parser;

use crate::commands::formatting::FormatArgs;
use crate::commands::formatting::FormatTool;
use crate::commands::outcome::CommandOutcome;
use crate::config::Configuration;
use crate::error::Error;

/// Format Twig syntax without rewriting HTML.
#[derive(Parser, Debug, Default)]
#[command(name = "twig:fmt", about = "Format Twig templates.")]
pub struct TwigFormatCommand {
    #[command(flatten)]
    pub(crate) arguments: FormatArgs,
}

impl TwigFormatCommand {
    pub fn execute(self, configuration: Configuration, color: ColorChoice) -> Result<CommandOutcome, Error> {
        self.arguments.execute(FormatTool::Twig, configuration, color)
    }
}
