use std::process::ExitCode;

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
mod database;
mod error;
mod macros;
mod pipeline;
mod utils;

#[cfg(all(not(feature = "dhat-heap"), any(target_os = "macos", target_os = "windows", target_env = "musl")))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

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

#[inline(always)]
pub fn run() -> Result<ExitCode, Error> {
    let arguments = CliArguments::parse();

    initialize_logger(
        if cfg!(debug_assertions) { LevelFilter::DEBUG } else { LevelFilter::INFO },
        "MAGO_LOG",
        arguments.colors.should_use_colors(),
    );

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

    #[cfg(not(unix))]
    if configuration.use_pager {
        tracing::warn!("The pager is only supported on unix-like systems. Ignoring the `use-pager` configuration.");
    }

    let use_colors = arguments.colors.should_use_colors();

    match command {
        MagoCommand::Init(cmd) => cmd.execute(configuration, None),
        MagoCommand::Config(cmd) => cmd.execute(configuration),
        MagoCommand::Lint(cmd) => cmd.execute(configuration, use_colors),
        MagoCommand::Format(cmd) => cmd.execute(configuration, use_colors),
        MagoCommand::Ast(cmd) => cmd.execute(configuration, use_colors),
        MagoCommand::Analyze(cmd) => cmd.execute(configuration, use_colors),
        MagoCommand::SelfUpdate(_) => {
            unreachable!("The self-update command should have been handled before this point.")
        }
    }
}
