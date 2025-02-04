use std::process::ExitCode;

use clap::Parser;
use tokio::runtime::Builder;
use tracing::level_filters::LevelFilter;

use crate::commands::MagoCommand;
use crate::config::Configuration;
use crate::consts::MAXIMUM_PHP_VERSION;
use crate::consts::MINIMUM_PHP_VERSION;
use crate::error::Error;
use crate::utils::logger::initialize_logger;

mod commands;
mod config;
mod consts;
mod error;
mod macros;
mod reflection;
mod source;
mod utils;

pub fn main() -> ExitCode {
    initialize_logger(if cfg!(debug_assertions) { LevelFilter::DEBUG } else { LevelFilter::INFO }, "MAGO_LOG");

    run().unwrap_or_else(|error| {
        tracing::error!("{}", error);

        ExitCode::FAILURE
    })
}

#[inline(always)]
pub fn run() -> Result<ExitCode, Error> {
    // Load the configuration.
    let configuration = Configuration::load()?;

    if configuration.php_version < MINIMUM_PHP_VERSION {
        return Err(Error::PHPVersionIsTooOld(MINIMUM_PHP_VERSION, configuration.php_version));
    }

    if configuration.php_version > MAXIMUM_PHP_VERSION {
        return Err(Error::PHPVersionIsTooNew(MAXIMUM_PHP_VERSION, configuration.php_version));
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

    match MagoCommand::parse() {
        MagoCommand::Lint(cmd) => runtime.block_on(commands::lint::execute(cmd, configuration)),
        MagoCommand::Fix(cmd) => runtime.block_on(commands::fix::execute(cmd, configuration)),
        MagoCommand::Format(cmd) => runtime.block_on(commands::format::execute(cmd, configuration)),
        MagoCommand::Ast(cmd) => runtime.block_on(commands::ast::execute(cmd)),
        MagoCommand::Find(find) => runtime.block_on(commands::find::execute(find, configuration)),
        MagoCommand::SelfUpdate(cmd) => commands::self_update::execute(cmd),
    }
}
