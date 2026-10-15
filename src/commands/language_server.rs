//! `mago language-server` — speaks the Language Server Protocol over stdio.
//!
//! Editors invoke this as a child process. The flags here let an editor
//! pick a trimmed-down profile when it only needs a subset of the server.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use tokio::runtime::Builder;

use crate::config::Configuration;
use crate::error::Error;
use crate::language_server::ServerConfig;

#[derive(Parser, Debug)]
#[command(
    name = "language-server",
    about = "Start the Mago language server (LSP over stdio).",
    long_about = indoc::indoc! {r"
        Start the Mago language server, speaking LSP over stdio. Editors
        invoke this as a child process and route their requests through it.

        Disable subsystems with --no-analyzer, --no-linter, or --no-formatter.
        Syntax checks and declaration scanning remain enabled. Disable individual
        providers through [language-server].disable in mago.toml.

        **NOTE**: The LSP is a work in progress. The set of advertised capabilities,
        the wire behaviour, the flags below, and even the existence of
        this subcommand can change or disappear without notice. There are
        no compatibility guarantees until mago 2.0. if you need a stable
        editor integration, wait for that release.
    "}
)]
pub struct LanguageServerCommand {
    /// Disable the static analyzer.
    ///
    /// Skip body analysis and type diagnostics. Keep declaration scanning,
    /// syntax checks, and basic navigation. Inferred member types are unavailable;
    /// signature help, type and call hierarchies, inlay hints, and code lenses
    /// are disabled.
    #[arg(long, default_value_t = false)]
    pub no_analyzer: bool,

    /// Disable the linter.
    ///
    /// Skip lint rules and their fixes. Syntax diagnostics and other code
    /// actions remain available, even when the analyzer is also disabled.
    #[arg(long, default_value_t = false)]
    pub no_linter: bool,

    /// Disable the formatter.
    ///
    /// Disable document and range formatting.
    #[arg(long, default_value_t = false)]
    pub no_formatter: bool,
}

impl LanguageServerCommand {
    pub fn execute(self, configuration: Configuration, workspace_override: Option<PathBuf>) -> Result<ExitCode, Error> {
        if std::env::var("MAGO_EXPERIMENTAL_SERVER").is_err() {
            tracing::error!(
                "The `language-server` subcommand is a work in progress. To run it, set the environment variable `MAGO_EXPERIMENTAL_SERVER=1` and try again."
            );
            tracing::error!(
                "The set of advertised capabilities, the wire behaviour, the flags below, and even the existence of this subcommand can change or disappear without notice."
            );
            tracing::error!(
                "There are no compatibility guarantees until mago 2.0. if you need a stable editor integration, wait for that release."
            );
            tracing::error!(
                "If you want to help shape the LSP, or if you have a specific editor integration in mind, please open an issue or a PR."
            );

            return Ok(ExitCode::FAILURE);
        } else {
            tracing::warn!(
                "The `language-server` subcommand is a work in progress. The set of advertised capabilities, the wire behaviour, the flags below, and even the existence of this subcommand can change or disappear without notice."
            );
            tracing::warn!(
                "There are no compatibility guarantees until mago 2.0. if you need a stable editor integration, wait for that release."
            );
            tracing::warn!(
                "If you want to help shape the LSP, or if you have a specific editor integration in mind, please open an issue or a PR."
            );
        }

        let config = ServerConfig {
            analyzer: !self.no_analyzer,
            linter: !self.no_linter,
            formatter: !self.no_formatter,
            configuration,
            workspace_override,
        };

        let runtime = Builder::new_multi_thread().enable_all().build().map_err(Error::BuildingRuntime)?;
        match runtime.block_on(crate::language_server::run(config)) {
            Ok(()) => Ok(ExitCode::SUCCESS),
            Err(error) => {
                tracing::error!(%error, "language server stopped");
                Ok(ExitCode::FAILURE)
            }
        }
    }
}
