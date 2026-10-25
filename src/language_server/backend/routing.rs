//! Route requests to workspace state and trace handlers.

use std::sync::Arc;
use std::time::Instant;

use tower_lsp_server::gen_lsp_types::Uri;
use tower_lsp_server::jsonrpc::Result as JsonRpcResult;

use mago_database::file::File as MagoFile;
use mago_server::workspace::document_path::DocumentPath;

use crate::language_server::registry::WorkspaceRegistry;
use crate::language_server::state::WorkspaceState;
use crate::language_server::uri::UriFilePath;

use super::Backend;

impl Backend {
    /// Run `f` against the whole registry (e.g. cross-workspace operations).
    pub(super) fn with_registry<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&WorkspaceRegistry) -> R,
    {
        Some(self.session.get()?.with_registry(f))
    }

    /// Run `f` against the whole registry, mutably.
    pub(super) fn with_registry_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut WorkspaceRegistry) -> R,
    {
        Some(self.session.get()?.with_registry_mut(f))
    }
}

/// Run a synchronous LSP capability handler with a tracing span attached
/// for telemetry. Slow handlers (≥50ms) log at `debug`; the rest at `trace`.
pub(super) fn traced<T, F>(name: impl std::fmt::Debug, f: F) -> JsonRpcResult<T>
where
    F: FnOnce() -> JsonRpcResult<T>,
{
    let started = Instant::now();
    let result = f();
    let elapsed = started.elapsed();
    if elapsed.as_millis() >= 50 {
        tracing::debug!(handler = ?name, elapsed = ?elapsed, "lsp handler");
    } else {
        tracing::trace!(handler = ?name, elapsed = ?elapsed, "lsp handler");
    }
    result
}

pub(super) fn file_for_uri(workspace: &WorkspaceState, uri: &Uri) -> Option<Arc<MagoFile>> {
    let path = uri.to_file_path()?;
    DocumentPath::new(&path).file(&workspace.core)
}
