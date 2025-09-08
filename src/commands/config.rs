use std::process::ExitCode;

use clap::Parser;
use clap::ValueEnum;

use crate::config::Configuration;
use crate::error::Error;

#[derive(ValueEnum, Debug, Clone, Copy)]
#[value(rename_all = "kebab-case")]
enum ConfigSection {
    Source,
    Linter,
    Formatter,
    Analyzer,
}

/// Display the final, merged configuration that Mago is using.
///
/// This command is useful for debugging your setup. It prints the fully resolved
/// configuration, showing the combined result of your `mago.toml` file, any
/// environment variables, and the built-in default values.
#[derive(Parser, Debug)]
#[command(name = "config", about, long_about)]
pub struct ConfigCommand {
    /// Display only a specific section of the configuration.
    #[arg(long, value_enum)]
    show: Option<ConfigSection>,
}

impl ConfigCommand {
    pub fn execute(self, configuration: Configuration) -> Result<ExitCode, Error> {
        let json = if let Some(section) = self.show {
            match section {
                ConfigSection::Source => serde_json::to_string_pretty(&configuration.source)?,
                ConfigSection::Linter => serde_json::to_string_pretty(&configuration.linter)?,
                ConfigSection::Formatter => serde_json::to_string_pretty(&configuration.formatter)?,
                ConfigSection::Analyzer => serde_json::to_string_pretty(&configuration.analyzer)?,
            }
        } else {
            serde_json::to_string_pretty(&configuration)?
        };

        println!("{}", json);

        Ok(ExitCode::SUCCESS)
    }
}
