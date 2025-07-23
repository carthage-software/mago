use std::process::ExitCode;

use clap::Parser;
use tokio::runtime::Builder;
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
mod metadata;
mod source;
mod utils;

#[cfg(any(target_os = "macos", target_os = "windows", target_env = "musl"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub fn main() -> ExitCode {
    initialize_logger(if cfg!(debug_assertions) { LevelFilter::DEBUG } else { LevelFilter::INFO }, "MAGO_LOG");

    run().unwrap_or_else(|error| {
        tracing::error!("{}", error);

        ExitCode::FAILURE
    })
}

#[inline(always)]
pub fn run() -> Result<ExitCode, Error> {
    let arguments = CliArguments::parse();
    if let MagoCommand::SelfUpdate(cmd) = arguments.command {
        return commands::self_update::execute(cmd);
    }

    let php_version = arguments.get_php_version()?;
    let CliArguments { workspace, config, threads, allow_unsupported_php_version, command, .. } = arguments;

    // Load the configuration.
    let configuration = Configuration::load(workspace, config, php_version, threads, allow_unsupported_php_version)?;

    if !configuration.allow_unsupported_php_version {
        if configuration.php_version < MINIMUM_PHP_VERSION {
            return Err(Error::PHPVersionIsTooOld(MINIMUM_PHP_VERSION, configuration.php_version));
        }

        if configuration.php_version > MAXIMUM_PHP_VERSION {
            return Err(Error::PHPVersionIsTooNew(MAXIMUM_PHP_VERSION, configuration.php_version));
        }
    }

    // Create the runtime.
    let runtime = if configuration.threads <= 1 {
        Builder::new_current_thread().enable_all().build().map_err(Error::BuildingRuntime)?
    } else {
        Builder::new_multi_thread()
            .worker_threads(configuration.threads)
            .thread_stack_size(configuration.stack_size)
            .enable_all()
            .build()
            .map_err(Error::BuildingRuntime)?
    };

    match command {
        MagoCommand::Init(cmd) => runtime.block_on(commands::init::execute(cmd, configuration)),
        MagoCommand::Lint(cmd) => runtime.block_on(commands::lint::execute(cmd, configuration)),
        MagoCommand::Format(cmd) => runtime.block_on(commands::format::execute(cmd, configuration)),
        MagoCommand::Ast(cmd) => runtime.block_on(commands::ast::execute(cmd)),
        MagoCommand::Analyze(cmd) => runtime.block_on(commands::analyze::execute(cmd, configuration)),
        MagoCommand::Find(cmd) => runtime.block_on(commands::find::execute(cmd, configuration)),
        MagoCommand::SelfUpdate(_) => {
            unreachable!("The self-update command should have been handled before this point.")
        }
    }
}
