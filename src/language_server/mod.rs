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

use std::io;
use std::sync::Arc;

use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tower_lsp_server::LspService;
use tower_lsp_server::Server;

mod backend;
mod capabilities;
mod codec;
pub(crate) mod config;
mod diagnostic_cache;
mod diagnostic_publication;
mod diagnostics;
mod ordered_service;
mod position;
mod registry;
mod sources;
mod state;
#[cfg(all(test, not(windows)))]
mod tests;
mod transport;
mod uri;
mod workspace;

pub use backend::Backend;
pub use config::ServerConfig;

/// Run the LSP server on stdin/stdout with the given configuration.
pub async fn run(config: ServerConfig) -> io::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    serve(stdin, stdout, config).await
}

/// Run the LSP server on the given async reader/writer.
pub async fn serve<R, W>(reader: R, writer: W, config: ServerConfig) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let config = Arc::new(config);
    let service = LspService::build(move |client| Backend::new(client, Arc::clone(&config)));
    #[cfg(test)]
    let service = service.custom_method("mago/testBarrier", Backend::test_barrier);
    let (service, socket) = service.finish();
    let service = ordered_service::OrderedService::new(service)?;
    let mut stopped = service.stopped();
    let session = service.session();
    let (failure, mut failed) = tokio::sync::watch::channel(None);
    let reader = transport::Observed::new(transport::FrameReader::new(reader), failure.clone());
    let writer = transport::Observed::new(writer, failure);
    tokio::select! {
        _ = Server::new(reader, writer, socket)
            .concurrency_level(ordered_service::REPLY_CONCURRENCY).serve(service) => {}
        _ = stopped.changed() => {}
        _ = failed.changed() => {}
    }
    match &*failed.borrow() {
        Some((kind, message)) => Err(io::Error::new(*kind, message.clone())),
        None => session.lock().unwrap().result(),
    }
}
