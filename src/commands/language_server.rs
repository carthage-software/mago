//! `mago language-server` — speaks the Language Server Protocol over stdio.
//!
//! Editors invoke this as a child process. The flags here let an editor
//! pick a trimmed-down profile when it only needs a subset of the server.

use std::process::ExitCode;

use clap::Parser;

use crate::error::Error;
use crate::language_server::ServerConfig;

#[derive(Parser, Debug)]
#[command(
    name = "language-server",
    about = "Start the Mago language server (LSP over stdio).",
    long_about = indoc::indoc! {r"
        Start the Mago language server, speaking LSP over stdio. Editors
        invoke this as a child process and route their requests through it.

        Each subsystem can be turned off when an editor doesn't need it,
        for a smaller footprint or a faster bootstrap. Pass
        --no-analyzer --no-formatter for a diagnostics-only profile.

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
    /// With the analyzer off, capabilities that depend on its artifacts
    /// degrade: hover renders only the symbol kind (no type signature),
    /// completion drops `$obj->`, `$obj?->`, and `Class::` member lookups,
    /// and signature help, inlay hints, and code lenses go quiet. Goto
    /// definition for class-likes, free functions, and constants still
    /// works (those resolve from prelude metadata).
    #[arg(long, default_value_t = false)]
    pub no_analyzer: bool,

    /// Disable the linter.
    ///
    /// No lint diagnostics are published, and `quickfix` code actions
    /// disappear since they're sourced from linter rules. If the analyzer
    /// is also off, the editor receives no diagnostics at all.
    #[arg(long, default_value_t = false)]
    pub no_linter: bool,

    /// Disable the formatter.
    ///
    /// `textDocument/formatting` is dropped from the advertised server
    /// capabilities, so editors won't offer "format document" for files
    /// served by mago.
    #[arg(long, default_value_t = false)]
    pub no_formatter: bool,
}

impl LanguageServerCommand {
    pub fn execute(self) -> Result<ExitCode, Error> {
        let config =
            ServerConfig { analyzer: !self.no_analyzer, linter: !self.no_linter, formatter: !self.no_formatter };

        let runtime =
            tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(Error::BuildingRuntime)?;
        runtime.block_on(crate::language_server::run(config));
        Ok(ExitCode::SUCCESS)
    }
}
