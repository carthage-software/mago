#![expect(deprecated)]

use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::builder::styling::Effects;

use mago_php_version::PHPVersion;

use crate::commands::ast::AstCommand;
use crate::commands::find::FindCommand;
use crate::commands::fix::FixCommand;
use crate::commands::format::FormatCommand;
use crate::commands::init::InitCommand;
use crate::commands::lint::LintCommand;
use crate::commands::self_update::SelfUpdateCommand;
use crate::error::Error;

pub mod ast;
pub mod find;
pub mod fix;
pub mod format;
pub mod init;
pub mod lint;
pub mod self_update;

/// Styling for the Mago CLI.
pub const CLAP_STYLING: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

/// The main Mago CLI command.
#[derive(Parser, Debug)]
pub enum MagoCommand {
    /// Initialize the configuration for Mago.
    #[command(name = "init")]
    Init(InitCommand),
    /// Analyze the abstract syntax tree (AST) of PHP code.
    #[command(name = "ast")]
    Ast(AstCommand),
    /// Lint PHP code using Mago's linter.
    #[command(name = "lint")]
    Lint(LintCommand),
    /// Automatically fix linting issues in PHP code (deprecated).
    #[command(name = "fix")]
    Fix(FixCommand),
    /// Format PHP code using Mago's formatter.
    #[command(name = "format")]
    Format(FormatCommand),
    /// Find references to symbols in PHP code.
    #[command(name = "find")]
    Find(FindCommand),
    /// Update Mago to the latest version.
    #[command(name = "self-update")]
    SelfUpdate(SelfUpdateCommand),
}

#[derive(Parser, Debug)]
#[command(
    version,
    author,
    styles = CLAP_STYLING,
    about = "Mago: The powerful PHP toolchain. Lint, format, find, and analyze your code with ease.",
    long_about = r#"
Welcome to Mago!

Mago is a powerful and versatile toolchain for PHP developers, designed to help you write better code, faster.

Features:

* **Linting:** Identify and fix code style issues and potential bugs.
* **Formatting:** Format your code consistently and automatically.
* **Finding:** Quickly locate symbols and references in your codebase.
* **Analyzing:** Analyze your code for structure, complexity, and dependencies.

Get started by exploring the commands below!
"#
)]
pub struct CliArguments {
    #[arg(
        long,
        help = "The path to the workspace directory. This is the root directory of your project. If not specified, defaults to the current working directory."
    )]
    pub workspace: Option<PathBuf>,

    #[arg(
        long,
        help = "The path to the configuration file. If not specified, Mago will search for a `mago.toml` file in the workspace directory."
    )]
    pub config: Option<PathBuf>,

    #[arg(
        long,
        help = "The PHP version to use for parsing and analysis. This should be a valid PHP version number (e.g., 8.0, 8.1). This value overrides the `php_version` setting in the configuration file and the `MAGO_PHP_VERSION` environment variable."
    )]
    pub php_version: Option<String>,

    #[arg(
        long,
        help = "The number of threads to use for linting and formatting. If not specified, Mago will use all available logical CPUs. This value overrides the `threads` setting in the configuration file and the `MAGO_THREADS` environment variable."
    )]
    pub threads: Option<usize>,

    #[arg(
        long,
        help = "Allow using an unsupported PHP version. This is not recommended, as it may lead to unexpected behavior. This value overrides the `allow_unsupported_php_version` setting in the configuration file and the `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION` environment variable.",
        default_value_t = false
    )]
    pub allow_unsupported_php_version: bool,

    /// The subcommand to execute.
    #[clap(subcommand)]
    pub command: MagoCommand,
}

impl CliArguments {
    /// Get the PHP version from the command-line arguments.
    ///
    /// This function parses the `php_version` argument and returns a `Result` containing the `PHPVersion`, or an `Error` if the version is invalid.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `PHPVersion`, or an `Error` if the version is invalid.
    pub fn get_php_version(&self) -> Result<Option<PHPVersion>, Error> {
        let Some(version) = &self.php_version else {
            return Ok(None);
        };

        match PHPVersion::from_str(version) {
            Ok(version) => Ok(Some(version)),
            Err(error) => Err(Error::InvalidPHPVersion(version.clone(), error)),
        }
    }
}
