use std::io;
use std::process::ExitCode;

use clap::CommandFactory;
use clap::Parser;
use clap_complete::Shell;
use clap_complete::generate;

use crate::commands::CliArguments;
use crate::error::Error;

#[derive(Parser, Debug)]
#[command(
    name = "generate-completions",
    about = "Generate shell completions",
    long_about = r#"
The `generate-completions` command generates shell completions for Mago for the given shell.
"#
)]
pub struct GenerateCompletionsCommand {
    /// Select the shell for which the completions are generated.
    pub shell: Shell,
}

impl GenerateCompletionsCommand {
    pub fn execute(self) -> Result<ExitCode, Error> {
        generate(self.shell, &mut CliArguments::command(), "mago", &mut io::stdout());

        Ok(ExitCode::SUCCESS)
    }
}
