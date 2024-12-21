use std::process::ExitCode;

use clap::Parser;
use tokio::runtime::Builder;

use mago_feedback::initialize_logger;
use mago_feedback::LevelFilter;

use crate::commands::MagoCommand;
use crate::config::Configuration;
use crate::error::Error;

mod commands;
mod config;
mod consts;
mod error;
mod macros;
mod reflection;
mod source;
mod utils;

pub fn main() -> Result<ExitCode, Error> {
    // Set up the logger.
    initialize_logger(if cfg!(debug_assertions) { LevelFilter::DEBUG } else { LevelFilter::INFO }, "MAGO_LOG");

    // Load the configuration.
    let configuration = Configuration::load()?;

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
        MagoCommand::SelfUpdate(cmd) => commands::self_update::execute(cmd),
    }
}
