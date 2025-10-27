//! Mago - The Oxidized PHP Toolchain
//!
//! A blazing fast linter, formatter, and static analyzer for PHP, written in Rust.
//!
//! # Architecture
//!
//! The CLI is organized into several layers:
//!
//! - **Command Layer** ([`commands`]): Command-line interface and argument parsing
//! - **Configuration Layer** ([`config`]): Loading and merging configuration from files and environment
//! - **Service Layer** ([`service`]): Business logic for analysis, linting, formatting, and guarding
//! - **Baseline Layer** ([`baseline`]): Issue baseline management for incremental adoption
//!
//! # Commands
//!
//! Mago provides several commands for different tasks:
//!
//! - `mago init`: Initialize a new Mago configuration file
//! - `mago config`: Display the current configuration
//! - `mago lint`: Run linting rules on PHP code
//! - `mago analyze`: Perform static analysis
//! - `mago format`: Format PHP code
//! - `mago guard`: Enforce architectural rules
//! - `mago ast`: Display the abstract syntax tree
//! - `mago list-files`: List all files that would be processed
//! - `mago self-update`: Update Mago to the latest version
//!
//! # Configuration
//!
//! Configuration is loaded from multiple sources in order of precedence:
//!
//! 1. Command-line arguments
//! 2. Environment variables (prefixed with `MAGO_`)
//! 3. `mago.toml` file in the project directory
//! 4. Global configuration in `~/.config/mago/config.toml`
//! 5. Built-in defaults
//!
//! # Error Handling
//!
//! The CLI uses a custom [`Error`] type that provides detailed error messages and appropriate
//! exit codes. All errors are logged using the [`tracing`] framework before exiting.

use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;
use tracing::level_filters::LevelFilter;

use crate::commands::CliArguments;
use crate::commands::MagoCommand;
use crate::config::Configuration;
use crate::consts::MAXIMUM_PHP_VERSION;
use crate::consts::MINIMUM_PHP_VERSION;
use crate::error::Error;
use crate::utils::logger::initialize_logger;

mod baseline;
mod commands;
mod config;
mod consts;
mod error;
mod macros;
mod service;
mod utils;

#[cfg(all(not(feature = "dhat-heap"), any(target_os = "macos", target_os = "windows", target_env = "musl")))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Entry point for the Mago CLI application.
///
/// This function initializes the heap profiler (if enabled), runs the main application logic,
/// and handles any errors by logging them and returning an appropriate exit code.
///
/// # Returns
///
/// - [`ExitCode::SUCCESS`] if the command completed successfully
/// - [`ExitCode::FAILURE`] if an error occurred during execution
///
/// # Error Handling
///
/// All errors are logged using the [`tracing`] framework with full error context before
/// exiting with a failure code.
pub fn main() -> ExitCode {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let result = run();

    result.unwrap_or_else(|error| {
        tracing::error!("{}", error);
        tracing::trace!("Exiting with error code due to: {:#?}", error);

        ExitCode::FAILURE
    })
}

/// Core application logic for the Mago CLI.
///
/// This function handles the complete lifecycle of a Mago command:
///
/// 1. **Parse Arguments**: Uses [`clap`] to parse command-line arguments
/// 2. **Initialize Logging**: Sets up the tracing subscriber based on environment and flags
/// 3. **Handle Self-Update**: Special case for the self-update command (no config needed)
/// 4. **Load Configuration**: Merges configuration from files, environment, and arguments
/// 5. **Validate PHP Version**: Ensures the configured PHP version is supported
/// 6. **Initialize Thread Pool**: Configures Rayon for parallel processing
/// 7. **Execute Command**: Dispatches to the appropriate command handler
///
/// # Returns
///
/// - `Ok(ExitCode)` if the command completed (success or expected failure)
/// - `Err(Error)` if an unexpected error occurred
///
/// # Errors
///
/// Returns [`Error`] for various failure conditions:
///
/// - Invalid command-line arguments
/// - Configuration file parsing errors
/// - Unsupported PHP version
/// - Thread pool initialization failure
/// - Command execution errors
#[inline(always)]
pub fn run() -> Result<ExitCode, Error> {
    let arguments = CliArguments::parse();

    let color_choice = if arguments.no_color { ColorChoice::Never } else { arguments.colors };

    initialize_logger(
        if cfg!(debug_assertions) { LevelFilter::DEBUG } else { LevelFilter::INFO },
        "MAGO_LOG",
        color_choice,
    );

    if arguments.no_color {
        tracing::warn!(
            "The `--no-color` option is deprecated and will be removed in a future release. Please use `--colors never` instead."
        );
    }

    if let MagoCommand::SelfUpdate(cmd) = arguments.command {
        return commands::self_update::execute(cmd);
    }

    let php_version = arguments.get_php_version()?;
    let CliArguments { workspace, config, threads, allow_unsupported_php_version, command, .. } = arguments;

    // Load the configuration.
    let configuration =
        Configuration::load(workspace, config.as_deref(), php_version, threads, allow_unsupported_php_version)?;

    if !configuration.allow_unsupported_php_version {
        if configuration.php_version < MINIMUM_PHP_VERSION {
            return Err(Error::PHPVersionIsTooOld(MINIMUM_PHP_VERSION, configuration.php_version));
        }

        if configuration.php_version > MAXIMUM_PHP_VERSION {
            return Err(Error::PHPVersionIsTooNew(MAXIMUM_PHP_VERSION, configuration.php_version));
        }
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(configuration.threads)
        .stack_size(configuration.stack_size)
        .build_global()?;

    match command {
        MagoCommand::Init(cmd) => cmd.execute(configuration, None),
        MagoCommand::Config(cmd) => cmd.execute(configuration),
        MagoCommand::ListFiles(cmd) => cmd.execute(configuration, color_choice),
        MagoCommand::Lint(cmd) => cmd.execute(configuration, color_choice),
        MagoCommand::Format(cmd) => cmd.execute(configuration, color_choice),
        MagoCommand::Ast(cmd) => cmd.execute(configuration, color_choice),
        MagoCommand::Analyze(cmd) => cmd.execute(configuration, color_choice),
        MagoCommand::Guard(cmd) => cmd.execute(configuration, color_choice),
        MagoCommand::SelfUpdate(_) => {
            unreachable!("The self-update command should have been handled before this point.")
        }
        MagoCommand::GenerateCompletions(cmd) => cmd.execute(),
    }
}
