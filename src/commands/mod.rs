//! Command-line interface definitions and argument parsing.
//!
//! This module defines the CLI structure for Mago using the [`clap`] crate. It provides
//! a unified interface for all Mago commands (lint, analyze, format, etc.) along with
//! global options that apply across all commands.
//!
//! # Architecture
//!
//! The CLI is structured in three layers:
//!
//! 1. **Top-Level Arguments** ([`CliArguments`]): Global options like workspace path,
//!    configuration file, PHP version, and color settings
//! 2. **Command Enum** ([`MagoCommand`]): The specific command to execute (lint, analyze, etc.)
//! 3. **Command-Specific Arguments**: Each command variant contains its own argument struct
//!    with command-specific options
//!
//! # Command Execution Flow
//!
//! 1. **Parse Arguments**: [`clap`] parses command-line arguments into [`CliArguments`]
//! 2. **Load Configuration**: Global options are used to load and merge configuration
//! 3. **Dispatch Command**: The [`MagoCommand`] variant dispatches to the appropriate
//!    command implementation
//! 4. **Execute**: Each command's `execute()` method performs the actual operation
//!
//! # Available Commands
//!
//! - **`init`** ([`InitCommand`]): Initialize a new Mago configuration file
//! - **`config`** ([`ConfigCommand`]): Display the effective configuration
//! - **`list-files`** ([`ListFilesCommand`]): List files that will be processed
//! - **`lint`** ([`LintCommand`]): Run linting rules on PHP code
//! - **`analyze`** ([`AnalyzeCommand`]): Perform static analysis
//! - **`format`** ([`FormatCommand`]): Format PHP code
//! - **`guard`** ([`GuardCommand`]): Enforce architectural rules
//! - **`ast`** ([`AstCommand`]): Display the abstract syntax tree
//! - **`self-update`** ([`SelfUpdateCommand`]): Update Mago to the latest version
//! - **`generate-completions`** ([`GenerateCompletionsCommand`]): Generate shell completions
//!
//! # Configuration Hierarchy
//!
//! Command-line arguments have the highest precedence in the configuration hierarchy:
//!
//! 1. **CLI Arguments** (highest precedence)
//! 2. Environment variables (prefixed with `MAGO_`)
//! 3. `mago.toml` in the workspace
//! 4. Global config at `~/.config/mago/config.toml`
//! 5. Built-in defaults (lowest precedence)

use std::path::PathBuf;
use std::str::FromStr;

use clap::ColorChoice;
use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::builder::styling::Effects;

use mago_php_version::PHPVersion;

use crate::commands::analyze::AnalyzeCommand;
use crate::commands::ast::AstCommand;
use crate::commands::config::ConfigCommand;
use crate::commands::format::FormatCommand;
use crate::commands::generate_completions::GenerateCompletionsCommand;
use crate::commands::guard::GuardCommand;
use crate::commands::init::InitCommand;
use crate::commands::lint::LintCommand;
use crate::commands::list_files::ListFilesCommand;
use crate::commands::self_update::SelfUpdateCommand;
use crate::error::Error;

mod args;

pub mod analyze;
pub mod ast;
pub mod config;
pub mod format;
pub mod generate_completions;
pub mod guard;
pub mod init;
pub mod lint;
pub mod list_files;
pub mod self_update;

/// ANSI color styling configuration for Mago's CLI output.
///
/// This constant defines the color scheme used by [`clap`] when rendering help messages,
/// error messages, and other CLI output. The styling enhances readability and provides
/// visual hierarchy in terminal output.
///
/// # Color Scheme
///
/// - **Headers and Usage**: Bold green
/// - **Literals and Valid Values**: Bold cyan
/// - **Placeholders**: Cyan (non-bold)
/// - **Errors**: Bold red
/// - **Invalid Values**: Bold yellow
///
/// These colors respect terminal color support and will automatically disable if the
/// terminal doesn't support ANSI escape codes.
pub const CLAP_STYLING: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

/// Enum representing all available Mago commands.
///
/// This enum is used by [`clap`] to dispatch to the appropriate command implementation
/// based on the subcommand provided by the user. Each variant contains the parsed
/// arguments specific to that command.
///
/// # Command Dispatch
///
/// Commands are dispatched in [`run()`](crate::run) using a match statement that calls
/// the `execute()` method on each command variant. Most commands follow a common pattern:
///
/// 1. Load the database using the orchestrator
/// 2. Create the appropriate service (lint, analyze, format, guard)
/// 3. Run the service and collect results
/// 4. Report results using the configured output format
///
/// # Special Cases
///
/// - **`self-update`**: Handled before configuration loading since it doesn't need
///   project configuration
/// - **`init`**: Interactive command that creates a new configuration file
/// - **`config`**: Diagnostic command that displays the effective configuration
/// - **`list-files`**: Diagnostic command that shows which files will be processed
#[derive(Parser, Debug)]
pub enum MagoCommand {
    /// Initialize a new Mago configuration file.
    ///
    /// Creates a new `mago.toml` configuration file in the workspace directory with
    /// default settings. This command is interactive and will prompt for various
    /// options if not provided via command-line arguments.
    ///
    /// **Usage**: `mago init`
    #[command(name = "init")]
    Init(InitCommand),

    /// Display the effective configuration.
    ///
    /// Shows the final, merged configuration that Mago is using after combining
    /// all configuration sources (CLI args, environment variables, config files,
    /// and defaults). Useful for debugging configuration issues.
    ///
    /// **Usage**: `mago config`
    #[command(name = "config")]
    Config(ConfigCommand),

    /// List all files that will be processed.
    ///
    /// Displays all PHP files that Mago will scan based on the current configuration's
    /// paths, includes, and excludes. Useful for verifying that the correct files are
    /// being targeted before running analysis commands.
    ///
    /// **Usage**: `mago list-files`
    #[command(name = "list-files")]
    ListFiles(ListFilesCommand),

    /// Display the abstract syntax tree (AST) of PHP code.
    ///
    /// Parses PHP code and displays its AST structure, useful for understanding how
    /// Mago interprets code structure and for debugging parser issues.
    ///
    /// **Usage**: `mago ast [FILES...]`
    #[command(name = "ast")]
    Ast(AstCommand),

    /// Run linting rules on PHP code.
    ///
    /// Checks PHP code against configured linting rules to identify style violations,
    /// code smells, and potential bugs. Results can be output in various formats
    /// including human-readable, JSON, and GitHub Actions.
    ///
    /// **Usage**: `mago lint [OPTIONS]`
    #[command(name = "lint")]
    Lint(LintCommand),

    /// Perform static analysis on PHP code.
    ///
    /// Runs deep static analysis including type checking, control flow analysis,
    /// and detection of logical errors. This is the most comprehensive analysis
    /// command and may take longer than linting.
    ///
    /// **Usage**: `mago analyze [OPTIONS]`
    #[command(name = "analyze")]
    Analyze(AnalyzeCommand),

    /// Enforce architectural rules and layer dependencies.
    ///
    /// Checks that code follows defined architectural constraints, such as ensuring
    /// that certain layers don't depend on others. Helps maintain clean architecture
    /// in larger codebases.
    ///
    /// **Usage**: `mago guard [OPTIONS]`
    #[command(name = "guard")]
    Guard(GuardCommand),

    /// Format PHP code according to style settings.
    ///
    /// Automatically formats PHP code to match the configured style preferences.
    /// Can check formatting (`--check`) or apply changes in-place (`--fix`).
    ///
    /// **Usage**: `mago format [OPTIONS]`
    #[command(name = "format")]
    Format(FormatCommand),

    /// Update Mago to the latest version.
    ///
    /// Downloads and installs the latest version of Mago from GitHub releases.
    /// This command doesn't require project configuration and is handled specially
    /// before configuration loading.
    ///
    /// **Usage**: `mago self-update`
    #[command(name = "self-update")]
    SelfUpdate(SelfUpdateCommand),

    /// Generate shell completions for the given shell
    ///
    /// **Usage**: `mago generate-completions`
    #[command(name = "generate-completions")]
    GenerateCompletions(GenerateCompletionsCommand),
}

/// Top-level CLI arguments parsed by [`clap`].
///
/// This struct contains global options that apply to all Mago commands, plus the
/// specific command to execute. Arguments defined here have the highest precedence
/// in the configuration hierarchy and will override settings from configuration files
/// and environment variables.
///
/// # Configuration Precedence
///
/// Values provided via CLI arguments override all other sources:
///
/// 1. **CLI Arguments** (this struct) - highest precedence
/// 2. **Environment Variables** (`MAGO_*` prefix)
/// 3. **Workspace Config** (`mago.toml`)
/// 4. **Global Config** (`~/.config/mago/config.toml`)
/// 5. **Defaults** - lowest precedence
///
/// # Usage Pattern
///
/// The CLI follows this pattern in [`run()`](crate::run):
///
/// 1. Parse arguments into [`CliArguments`] using [`Parser::parse()`]
/// 2. Initialize logging based on debug mode and color settings
/// 3. Handle `self-update` command specially (no config needed)
/// 4. Load and merge configuration from all sources
/// 5. Validate PHP version if not explicitly allowed
/// 6. Initialize thread pool with configured settings
/// 7. Dispatch to the appropriate command's `execute()` method
///
/// # Color Output
///
/// The CLI supports multiple color modes via `--colors` or the deprecated `--no-color`.
/// Colors are automatically disabled when output is piped or when the terminal doesn't
/// support ANSI escape codes.
#[derive(Parser, Debug)]
#[command(
    version,
    author,
    styles = CLAP_STYLING,
    about = "Mago: The powerful PHP toolchain. Lint, format, and analyze your code with ease.",
    long_about = r#"
Welcome to Mago!

Mago is a powerful and versatile toolchain for PHP developers, designed to help you write better code, faster.

Features:

* **Linting:** Identify and fix code style issues and potential bugs.
* **Formatting:** Format your code consistently and automatically.
* **Analyzing:** Analyze your code for structure, complexity, and dependencies.
* **AST Inspection:** Dive deep into the structure of your PHP code with Abstract Syntax Tree (AST) visualization.

Get started by exploring the commands below!
"#
)]
pub struct CliArguments {
    /// The path to the workspace directory.
    ///
    /// This is the root directory of your project. If not specified, defaults to the current working directory.
    #[arg(long)]
    pub workspace: Option<PathBuf>,

    /// The path to the configuration file.
    ///
    /// This is the path to your `mago.toml` configuration file. If not specified, Mago will search for a `mago.toml` file in the workspace directory.
    /// If no configuration file is found, Mago will use default settings.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// The PHP version to use for parsing and analysis.
    ///
    /// This should be a valid PHP version number (e.g., 8.0, 8.1).
    /// This value overrides the `php-version` setting in the configuration file and the `MAGO_PHP_VERSION` environment variable.
    #[arg(long)]
    pub php_version: Option<String>,

    /// The number of threads to use for linting, formatting, and analysis.
    ///
    /// If not specified, Mago will use all available logical CPUs.
    /// This value overrides the `threads` setting in the configuration file and the `MAGO_THREADS` environment variable.
    #[arg(long)]
    pub threads: Option<usize>,

    /// Allow using an unsupported PHP version.
    ///
    /// Use this flag to bypass the check for supported PHP versions. This is not recommended, as it may lead to unexpected behavior.
    #[arg(long, default_value_t = false)]
    pub allow_unsupported_php_version: bool,

    /// Do not use colors in the output.
    ///
    /// This flag has been deprecated in favor of `--colors=never`.
    /// It will be removed in a future release.
    #[arg(long, default_value_t = false, alias = "no-colors")]
    pub no_color: bool,

    /// When to use colored output. Can be "auto", "always", or "never".
    ///
    /// - "auto": Use colors if the output is a terminal (default).
    /// - "always": Always use colors, even if the output is not a terminal.
    /// - "never": Never use colors.
    #[arg(long, default_value_t = ColorChoice::Auto, conflicts_with = "no_color")]
    pub colors: ColorChoice,

    /// The subcommand to execute.
    ///
    /// Use `mago <command> --help` to see detailed usage information for each command.
    #[clap(subcommand)]
    pub command: MagoCommand,
}

impl CliArguments {
    /// Parses the PHP version from command-line arguments.
    ///
    /// This method extracts and validates the `--php-version` argument if provided.
    /// The version string is parsed into a [`PHPVersion`] enum value that determines
    /// which language features are recognized during parsing and analysis.
    ///
    /// # Behavior
    ///
    /// - If `--php-version` is not provided, returns `Ok(None)`
    /// - If provided, attempts to parse the version string
    /// - Returns an error if the version string is invalid
    ///
    /// # Configuration Integration
    ///
    /// This method is called during configuration loading in [`run()`](crate::run).
    /// If a version is provided via CLI, it overrides the version from:
    /// - `MAGO_PHP_VERSION` environment variable
    /// - `php-version` in `mago.toml`
    /// - PHP version detected from `composer.json`
    /// - Default version
    ///
    /// # Returns
    ///
    /// - `Ok(Some(version))` if a valid version was provided
    /// - `Ok(None)` if no version was provided
    /// - `Err(Error::InvalidPHPVersion)` if the version string is malformed
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPHPVersion`] when the version string cannot be parsed.
    /// Common causes include:
    /// - Invalid format (e.g., "8.x" instead of "8.0")
    /// - Non-existent version numbers (e.g., "9.0")
    /// - Malformed strings (e.g., "eight")
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
