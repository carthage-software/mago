//! Configuration display command implementation.
//!
//! This module implements the `mago config` command, which displays the effective
//! configuration after merging all sources (files, environment variables, defaults).
//! This is a diagnostic tool essential for debugging configuration issues.
//!
//! # Configuration Merging
//!
//! Mago loads configuration from multiple sources in this precedence order:
//!
//! 1. **Command-line arguments** (highest precedence)
//! 2. **Environment variables** (prefixed with `MAGO_`)
//! 3. **Project config** (`./mago.toml`)
//! 4. **Global config** (`~/.config/mago/config.toml`)
//! 5. **Built-in defaults** (lowest precedence)
//!
//! # Output Format
//!
//! The command outputs configuration as JSON for easy parsing and readability.
//! Users can focus on specific sections or view defaults without any config files.
//!
//! # Use Cases
//!
//! - **Debugging**: Verify which settings are actually being used
//! - **Documentation**: See all available configuration options
//! - **CI Integration**: Extract configuration programmatically
//! - **Validation**: Confirm environment variables are being read correctly

use std::process::ExitCode;

use clap::Parser;
use clap::ValueEnum;

use crate::config::Configuration;
use crate::config::analyzer::AnalyzerConfiguration;
use crate::config::formatter::FormatterConfiguration;
use crate::config::linter::LinterConfiguration;
use crate::config::source::SourceConfiguration;
use crate::consts::CURRENT_DIR;
use crate::error::Error;

/// Configuration sections that can be displayed individually.
///
/// This enum represents the major configuration sections in Mago's configuration
/// file. Each section configures a different aspect of Mago's behavior.
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
#[command(
    name = "config",
    about = "Display the current configuration that Mago is using.",
    long_about = "Display the final, merged configuration that Mago is using for this project.\n\n\
                  This command is invaluable for debugging your setup. It shows you the\n\
                  complete configuration that results from combining:\n\
                  • Built-in default values\n\
                  • Global configuration from ~/.mago.toml or $XDG_CONFIG_HOME/mago.toml\n\
                  • Project configuration from ./mago.toml\n\
                  • Environment variables (MAGO_*)\n\
                  • Command-line overrides\n\n\
                  Use --show to focus on a specific section, or --default to see\n\
                  what the defaults would be without any configuration files."
)]
pub struct ConfigCommand {
    /// Display only a specific section of the configuration.
    ///
    /// Instead of showing the entire configuration, focus on just one section.
    ///
    /// Available sections: source, linter, formatter, analyzer.
    #[arg(long, value_enum)]
    show: Option<ConfigSection>,

    /// Show the default configuration values instead of the current ones.
    ///
    /// This ignores any configuration files and environment variables,
    /// showing only the built-in defaults that Mago would use if no
    /// configuration was provided.
    #[arg(long, default_value_t = false)]
    default: bool,
}

impl ConfigCommand {
    /// Executes the config display command.
    ///
    /// This method outputs the effective configuration as JSON, either for the
    /// entire configuration or a specific section. The output includes all merged
    /// settings from all sources unless `--default` is specified.
    ///
    /// # Behavior
    ///
    /// - If `--default` is set, shows built-in defaults only
    /// - If `--show <section>` is set, displays only that section
    /// - Otherwise, displays the complete merged configuration
    ///
    /// # Arguments
    ///
    /// * `configuration` - The fully merged configuration to display
    ///
    /// # Returns
    ///
    /// Always returns `Ok(ExitCode::SUCCESS)` since displaying configuration
    /// cannot fail (serialization to JSON is infallible for these types).
    ///
    /// # Output
    ///
    /// The configuration is serialized to pretty-printed JSON and written to stdout.
    /// This format is both human-readable and machine-parseable.
    pub fn execute(self, configuration: Configuration) -> Result<ExitCode, Error> {
        let json = if let Some(section) = self.show {
            match section {
                ConfigSection::Source => {
                    if self.default {
                        serde_json::to_string_pretty(&SourceConfiguration::from_workspace(CURRENT_DIR.clone()))?
                    } else {
                        serde_json::to_string_pretty(&configuration.source)?
                    }
                }
                ConfigSection::Linter => {
                    if self.default {
                        serde_json::to_string_pretty(&LinterConfiguration::default())?
                    } else {
                        serde_json::to_string_pretty(&configuration.linter)?
                    }
                }
                ConfigSection::Formatter => {
                    if self.default {
                        serde_json::to_string_pretty(&FormatterConfiguration::default())?
                    } else {
                        serde_json::to_string_pretty(&configuration.formatter)?
                    }
                }
                ConfigSection::Analyzer => {
                    if self.default {
                        serde_json::to_string_pretty(&AnalyzerConfiguration::default())?
                    } else {
                        serde_json::to_string_pretty(&configuration.analyzer)?
                    }
                }
            }
        } else if self.default {
            serde_json::to_string_pretty(&Configuration::from_workspace(CURRENT_DIR.clone()))?
        } else {
            serde_json::to_string_pretty(&configuration)?
        };

        println!("{}", json);

        Ok(ExitCode::SUCCESS)
    }
}
