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
//! - `mago generate-completions`: Generate shell completion scripts
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
use std::time::Duration;
use std::time::Instant;

use clap::Parser;
use tracing::Level;
use tracing::enabled;
use tracing::level_filters::LevelFilter;
use tracing::trace;

use crate::commands::CliArguments;
use crate::commands::MagoCommand;
use crate::config::Configuration;
use crate::consts::MAXIMUM_PHP_VERSION;
use crate::consts::MINIMUM_PHP_VERSION;
use crate::consts::VERSION;
use crate::error::Error;
use crate::utils::configure_colors;
use crate::utils::logger::initialize_logger;
use crate::version_check::VersionCheck;
use crate::version_check::VersionPin;

mod baseline;
mod commands;
mod config;
mod consts;
mod error;
#[cfg(feature = "language-server")]
mod language_server;
mod macros;
mod service;
mod updater;
mod utils;
mod version_check;

#[cfg(all(
    not(feature = "dhat-heap"),
    any(target_os = "macos", target_os = "windows", target_env = "musl", target_env = "gnu")
))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Exit code for tool errors (configuration errors, parse errors, etc.)
///
/// This is distinct from `ExitCode::FAILURE` (1) which indicates issues were found.
/// Exit code 2 indicates the tool itself failed to complete its operation.
const EXIT_CODE_ERROR: u8 = 2;

/// Entry point for the Mago CLI application.
///
/// This function initializes the heap profiler (if enabled), runs the main application logic,
/// and handles any errors by logging them and returning an appropriate exit code.
///
/// # Exit Codes
///
/// - `0` ([`ExitCode::SUCCESS`]): Command completed successfully with no issues found
/// - `1` ([`ExitCode::FAILURE`]): Issues were found at or above the minimum fail level
/// - `2` ([`EXIT_CODE_ERROR`]): Tool error occurred (configuration, parsing, I/O, etc.)
///
/// # Error Handling
///
/// All errors are logged using the [`tracing`] framework with full error context before
/// exiting with exit code 2.
pub fn main() -> ExitCode {
    // Captured before anything else so the elapsed counter spans the whole
    // process lifetime. The logger isn't initialized yet, so the eventual
    // trace event is emitted from inside `run` once tracing is wired up;
    // telemetry in this function only matters when `MAGO_LOG=trace`.
    let main_start = Instant::now();

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let code = run(main_start).unwrap_or_else(|error| {
        tracing::error!("{}", error);
        tracing::trace!("Exiting with error code due to: {:#?}", error);

        ExitCode::from(EXIT_CODE_ERROR)
    });

    // Note: this trace event may not actually emit because destructors of the
    // tracing subscriber run at process exit. It's still useful when the
    // subscriber is configured to flush eagerly.
    if enabled!(Level::TRACE) {
        trace!("total process time: {:?}", main_start.elapsed());
    }

    code
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
pub fn run(main_start: Instant) -> Result<ExitCode, Error> {
    // The tracing subscriber isn't set up until `initialize_logger` runs, so
    // any `enabled!(Level::TRACE)` checks *before* that point resolve against
    // the default (off) subscriber and elide their work as intended. Once the
    // logger is configured, subsequent checks in this function correctly
    // reflect the user's `MAGO_LOG` setting.

    let clap_start = Instant::now();
    let arguments = CliArguments::parse();
    let clap_duration = clap_start.elapsed();

    // Configure global color settings based on the color choice.
    // This must be done before initializing the logger or any other
    // component that uses colors.
    configure_colors(arguments.colors);

    let logger_start = Instant::now();
    initialize_logger(
        if cfg!(debug_assertions) { LevelFilter::DEBUG } else { LevelFilter::INFO },
        "MAGO_LOG",
        arguments.colors,
    );
    let logger_duration = logger_start.elapsed();

    // From here on `enabled!(Level::TRACE)` reflects the real subscriber.
    let trace_enabled = enabled!(Level::TRACE);
    let pre_logger_elapsed =
        if trace_enabled { main_start.elapsed() - clap_duration - logger_duration } else { Duration::ZERO };

    let php_version = arguments.get_php_version()?;
    let CliArguments { workspace, config, threads, allow_unsupported_php_version, no_version_check, command, .. } =
        arguments;

    let config_load_start = trace_enabled.then(Instant::now);
    let configuration = Configuration::load(
        workspace,
        config.as_deref(),
        php_version,
        threads,
        allow_unsupported_php_version,
        no_version_check,
    )?;
    let config_load_duration = config_load_start.map(|s| s.elapsed()).unwrap_or_default();

    if let MagoCommand::SelfUpdate(cmd) = command {
        return commands::self_update::execute(cmd, configuration.version);
    }

    check_project_version(&configuration)?;

    if !configuration.allow_unsupported_php_version {
        if configuration.php_version < MINIMUM_PHP_VERSION {
            return Err(Error::PHPVersionIsTooOld(MINIMUM_PHP_VERSION, configuration.php_version));
        }

        if configuration.php_version > MAXIMUM_PHP_VERSION {
            return Err(Error::PHPVersionIsTooNew(MAXIMUM_PHP_VERSION, configuration.php_version));
        }
    }

    let rayon_init_start = trace_enabled.then(Instant::now);
    rayon::ThreadPoolBuilder::new()
        .num_threads(configuration.threads)
        .stack_size(configuration.stack_size)
        .build_global()?;
    let rayon_init_duration = rayon_init_start.map(|s| s.elapsed()).unwrap_or_default();

    if trace_enabled {
        trace!("Process startup (dyld and runtime init) took {:?}.", pre_logger_elapsed);
        trace!("CLI arguments parsed in {:?}.", clap_duration);
        trace!("Logger initialized in {:?}.", logger_duration);
        trace!("Configuration loaded in {:?}.", config_load_duration);
        trace!("Rayon thread pool built in {:?}.", rayon_init_duration);
        trace!("Ready to dispatch command after {:?}.", main_start.elapsed());
    }

    let command_start = trace_enabled.then(Instant::now);
    let result = match command {
        MagoCommand::Init(cmd) => cmd.execute(configuration, None),
        MagoCommand::Config(cmd) => cmd.execute(configuration),
        MagoCommand::ListFiles(cmd) => cmd.execute(configuration, arguments.colors),
        MagoCommand::Lint(cmd) => cmd.execute(configuration, arguments.colors),
        MagoCommand::Format(cmd) => cmd.execute(configuration, arguments.colors),
        MagoCommand::Ast(cmd) => cmd.execute(configuration, arguments.colors),
        MagoCommand::Analyze(cmd) => cmd.execute(configuration, arguments.colors),
        MagoCommand::Guard(cmd) => cmd.execute(configuration, arguments.colors),
        MagoCommand::GenerateCompletions(cmd) => cmd.execute(),
        #[cfg(feature = "language-server")]
        MagoCommand::LanguageServer(cmd) => cmd.execute(configuration),
        MagoCommand::SelfUpdate(_) => {
            unreachable!("The self-update command should have been handled before this point.")
        }
    };

    if let Some(start) = command_start {
        trace!("Command finished in {:?}.", start.elapsed());
        trace!("Total time spent inside main so far: {:?}.", main_start.elapsed());
    }

    result
}

/// Verifies that the installed mago binary satisfies the `version` pin in
/// `mago.toml`, if one is set.
fn check_project_version(configuration: &Configuration) -> Result<(), Error> {
    let Some(pin_string) = configuration.version.as_deref() else {
        // TODO(azjezz): we should start emitting a warning here when nearing version 2.0
        return Ok(());
    };

    let pin = VersionPin::parse(pin_string)?;
    let result = pin.check(VERSION)?;

    match result {
        VersionCheck::Match => Ok(()),
        VersionCheck::MajorDrift => {
            let installed_major = VERSION.split('.').next().unwrap_or(VERSION);
            tracing::error!("Major versions may have incompatible config schemas; refusing to run.");
            tracing::error!("Run `mago self-update --to-project-version` to sync to the pinned major.");
            tracing::error!(
                "Or reinstall the matching binary, or bump `version` in mago.toml to `{installed_major}` once you have reviewed the changelog."
            );

            Err(Error::ProjectMajorVersionMismatch(pin.to_string(), VERSION.to_string()))
        }
        VersionCheck::MinorDrift | VersionCheck::PatchDrift => {
            if !configuration.no_version_check {
                tracing::warn!("mago.toml is pinned to `{pin}` but the installed mago binary is `{VERSION}`.");
                tracing::warn!("Run `mago self-update --to-project-version` to sync.");
                tracing::warn!("Pass `--no-version-check` or set `MAGO_NO_VERSION_CHECK=1` to silence this warning.");
            }

            Ok(())
        }
    }
}
