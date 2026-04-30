//! Language Server Protocol implementation for Mago.
//!
//! Surfaced as the (hidden) `mago language-server` subcommand. The
//! implementation wraps mago's analyzer, linter, and formatter behind a
//! `tower-lsp` dispatch and a long-lived `IncrementalAnalysisService`.
//!
//! # Entry points
//!
//! - [`run`]; speak LSP over `stdin`/`stdout`. Used by the CLI.
//! - [`serve`]; speak LSP over an arbitrary async reader/writer pair.
//!   Used by integration tests via `tokio::io::duplex`.

use std::sync::Arc;

use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tower_lsp::LspService;
use tower_lsp::Server;

mod backend;
mod capabilities;
mod config;
mod diagnostics;
mod document;
mod file_analysis;
mod linter;
mod position;
mod state;
mod workspace;

#[cfg(test)]
mod tests;

pub use backend::Backend;
pub use config::ServerConfig;

/// Run the LSP server on stdin/stdout with the given configuration.
pub async fn run(config: ServerConfig) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    serve(stdin, stdout, config).await;
}

/// Run the LSP server on the given async reader/writer.
pub async fn serve<R, W>(reader: R, writer: W, config: ServerConfig)
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let config = Arc::new(config);
    let (service, socket) = LspService::new(move |client| Backend::new(client, Arc::clone(&config)));
    Server::new(reader, writer, socket).serve(service).await;
}
