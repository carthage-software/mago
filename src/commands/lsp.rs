use std::process::ExitCode;

use clap::Parser;
use tower_lsp::LspService;
use tower_lsp::Server;

use mago_interner::ThreadedInterner;

use crate::error::Error;
use crate::lsp::MagoLanguageServer;

/// Represents the `lsp` command, which starts the Language Server Protocol (LSP) server.
#[derive(Parser, Debug)]
#[command(
    name = "lsp",
    about = "Starts the Language Server Protocol (LSP) server.",
    long_about = r#"
The `lsp` command starts the Language Server Protocol (LSP) server, which can be used to provide
editor support for PHP files. The server will listen for incoming connections on stdin and stdout.
"#
)]
pub struct LspCommand {}

/// Executes the LSP command with the provided options.
///
/// # Arguments
///
/// * `command` - The `LspCommand` structure containing user-specified options.
///
/// # Returns
///
/// An `ExitCode` indicating the success or failure of the command.
pub async fn execute(_command: LspCommand) -> Result<ExitCode, Error> {
    let interner = ThreadedInterner::new();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(|client| MagoLanguageServer::new(interner, client));
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(ExitCode::SUCCESS)
}
