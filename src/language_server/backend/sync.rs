//! Helpers attached to [`super::Backend`] that don't belong in the
//! `tower-lsp` trait dispatch: lifecycle (`bootstrap`), workspace mutation
//! (`apply_change_atomic`), and shared utilities.

use std::borrow::Cow;
use std::sync::Arc;

use foldhash::HashMap;
use tower_lsp::jsonrpc::Result as JsonRpcResult;
use tower_lsp::lsp_types::Diagnostic;
use tower_lsp::lsp_types::MessageType;
use tower_lsp::lsp_types::Url;

use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_database::file::FileId;
use mago_database::file::FileType;

use crate::language_server::diagnostics::build_diagnostics;
use crate::language_server::document::OpenDocument;
use crate::language_server::state::BackendState;
use crate::language_server::state::WorkspaceState;
use crate::language_server::state::build_workspace;
use crate::language_server::workspace::file_id_for;
use crate::language_server::workspace::logical_name_for;

use super::Backend;

impl Backend {
    pub(super) async fn bootstrap(&self) {
        let pending = match std::mem::replace(&mut *self.state.lock().unwrap(), BackendState::Uninitialized) {
            BackendState::Pending(config) => config,
            other => {
                *self.state.lock().unwrap() = other;
                return;
            }
        };

        let started = std::time::Instant::now();
        tracing::info!(root = %pending.root.display(), "bootstrap starting");

        let outcome = tokio::task::spawn_blocking(move || build_workspace(pending)).await;

        match outcome {
            Ok(Ok((mut workspace, analysis_result))) => {
                tracing::info!(
                    elapsed = ?started.elapsed(),
                    issues = analysis_result.issues.len(),
                    "analyzer pass complete",
                );

                if workspace.config.linter {
                    let lint_started = std::time::Instant::now();
                    analyze_all_workspace_files(&mut workspace);
                    tracing::info!(elapsed = ?lint_started.elapsed(), "file analysis pass complete");
                }

                let lint_issues = workspace.file_analyses.values().map(|(_, a)| &a.lint_issues);
                let diagnostics = build_diagnostics(&workspace.database, &analysis_result, lint_issues);
                let publish_count = diagnostics.values().map(|v| v.len()).sum::<usize>();
                *self.state.lock().unwrap() = BackendState::Ready(workspace);
                self.publish(diagnostics).await;
                tracing::info!(elapsed = ?started.elapsed(), diagnostics = publish_count, "ready");
            }
            Ok(Err(err)) => {
                tracing::error!(error = %err, "bootstrap failed");
                self.client.log_message(MessageType::ERROR, format!("mago-server: bootstrap failed: {err}")).await;
            }
            Err(err) => {
                tracing::error!(error = %err, "bootstrap task panicked");
                self.client
                    .log_message(MessageType::ERROR, format!("mago-server: bootstrap task panicked: {err}"))
                    .await;
            }
        }
    }

    /// Apply a database mutation under the workspace mutex AND immediately
    /// run incremental analysis on the affected files; all without
    /// dropping the lock in between.
    ///
    /// This prevents capability handlers (which also acquire the mutex)
    /// from observing a database with new contents but stale analysis
    /// while the analyzer thread is in flight.
    pub(super) async fn apply_change_atomic<F>(&self, mutate: F)
    where
        F: FnOnce(&mut WorkspaceState) -> Vec<FileId> + Send + 'static,
    {
        let started = std::time::Instant::now();
        let state = Arc::clone(&self.state);
        let outcome = tokio::task::spawn_blocking(move || -> Result<_, mago_orchestrator::error::OrchestratorError> {
            let mut guard = state.lock().unwrap();
            let BackendState::Ready(workspace) = &mut *guard else {
                return Ok(None);
            };

            let changed = mutate(workspace);
            if changed.is_empty() {
                return Ok(None);
            }

            workspace.service.update_database(workspace.database.read_only());
            let result = if workspace.config.analyzer {
                workspace.service.analyze_incremental(Some(&changed))?
            } else {
                mago_analyzer::analysis_result::AnalysisResult::new(mago_codex::reference::SymbolReferences::new())
            };
            workspace.invalidate_artifacts(&changed);
            crate::language_server::capabilities::lookup::invalidate(&changed);

            if workspace.config.linter {
                workspace.refresh_analyses(&changed);
            }

            let lint_issues = workspace.file_analyses.values().map(|(_, a)| &a.lint_issues);
            let diagnostics = build_diagnostics(&workspace.database, &result, lint_issues);
            Ok(Some((changed.len(), diagnostics)))
        })
        .await;

        match outcome {
            Ok(Ok(Some((count, diagnostics)))) => {
                tracing::debug!(files = count, elapsed = ?started.elapsed(), "incremental analysis");
                self.publish(diagnostics).await;
            }
            Ok(Ok(None)) => {}
            Ok(Err(err)) => {
                tracing::error!(error = %err, "incremental analysis failed");
                self.client
                    .log_message(MessageType::ERROR, format!("mago-server: incremental analysis failed: {err}"))
                    .await;
            }
            Err(err) => {
                tracing::error!(error = %err, "analysis task panicked");
                self.client
                    .log_message(MessageType::ERROR, format!("mago-server: analysis task panicked: {err}"))
                    .await;
            }
        }
    }

    pub(super) async fn apply_buffer_open(&self, uri: Url, text: String, version: i32) {
        let Ok(path) = uri.to_file_path() else {
            return;
        };
        self.apply_change_atomic(move |workspace| {
            let logical = logical_name_for(&workspace.root, &path);
            let virtual_file = workspace.database.get_id(&logical).is_none();
            let file_id = if virtual_file {
                let file =
                    MagoFile::new(Cow::Owned(logical.clone()), FileType::Host, Some(path.clone()), Cow::Owned(text));
                workspace.database.add(file)
            } else {
                let id = FileId::new(&logical);
                workspace.database.update(id, Cow::Owned(text));
                id
            };
            workspace.open_documents.insert(uri, OpenDocument { file_id, virtual_file, version });
            vec![file_id]
        })
        .await;
    }

    pub(super) async fn apply_buffer_change(&self, uri: Url, text: String, version: i32) {
        self.apply_change_atomic(move |workspace| {
            let Some(open) = workspace.open_documents.get_mut(&uri) else {
                return Vec::new();
            };
            open.version = version;
            workspace.database.update(open.file_id, Cow::Owned(text));
            vec![open.file_id]
        })
        .await;
    }

    pub(super) async fn apply_buffer_close(&self, uri: Url) {
        let Ok(path) = uri.to_file_path() else {
            return;
        };
        self.apply_change_atomic(move |workspace| {
            let Some(open) = workspace.open_documents.remove(&uri) else {
                return Vec::new();
            };
            if open.virtual_file {
                workspace.database.delete(open.file_id);
            } else if let Ok(file) = MagoFile::read(&workspace.root, &path, FileType::Host) {
                workspace.database.update(open.file_id, file.contents);
            }
            vec![open.file_id]
        })
        .await;
    }

    pub(super) async fn apply_disk_change(&self, uri: Url) {
        let Ok(path) = uri.to_file_path() else {
            return;
        };
        self.apply_change_atomic(move |workspace| {
            if workspace.open_documents.contains_key(&uri) {
                return Vec::new();
            }
            let Ok(file) = MagoFile::read(&workspace.root, &path, FileType::Host) else {
                return Vec::new();
            };
            let id = file.id;
            if workspace.database.get(&id).is_ok() {
                workspace.database.update(id, file.contents);
            } else {
                workspace.database.add(file);
            }
            vec![id]
        })
        .await;
    }

    pub(super) async fn apply_disk_delete(&self, uri: Url) {
        let Ok(path) = uri.to_file_path() else {
            return;
        };
        self.apply_change_atomic(move |workspace| {
            let id = file_id_for(&workspace.root, &path);
            if workspace.database.delete(id) { vec![id] } else { Vec::new() }
        })
        .await;
    }

    pub(super) fn with_file<F, R>(&self, uri: &Url, f: F) -> Option<R>
    where
        F: FnOnce(&MagoFile, &WorkspaceState) -> R,
    {
        let guard = self.state.lock().unwrap();
        let BackendState::Ready(workspace) = &*guard else {
            return None;
        };
        let file = file_for_uri(workspace, uri)?;
        Some(f(&file, workspace))
    }

    pub(super) fn with_workspace<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&WorkspaceState) -> R,
    {
        let guard = self.state.lock().unwrap();
        let BackendState::Ready(workspace) = &*guard else {
            return None;
        };
        Some(f(workspace))
    }

    pub(super) fn with_workspace_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut WorkspaceState) -> R,
    {
        let mut guard = self.state.lock().unwrap();
        let BackendState::Ready(workspace) = &mut *guard else {
            return None;
        };
        Some(f(workspace))
    }

    async fn publish(&self, diagnostics: HashMap<Url, Vec<Diagnostic>>) {
        let stale: Vec<Url> = {
            let mut guard = self.state.lock().unwrap();
            if let BackendState::Ready(workspace) = &mut *guard {
                let stale: Vec<Url> =
                    workspace.last_diagnostics.keys().filter(|url| !diagnostics.contains_key(*url)).cloned().collect();
                workspace.last_diagnostics = diagnostics.clone();
                stale
            } else {
                return;
            }
        };

        for url in stale {
            self.client.publish_diagnostics(url, vec![], None).await;
        }
        for (url, diags) in diagnostics {
            self.client.publish_diagnostics(url, diags, None).await;
        }
    }
}

/// Run a synchronous LSP capability handler with a tracing span attached
/// for telemetry. Slow handlers (≥50ms) log at `debug`; the rest at `trace`.
pub(super) fn traced<T, F>(name: &'static str, f: F) -> JsonRpcResult<T>
where
    F: FnOnce() -> JsonRpcResult<T>,
{
    let started = std::time::Instant::now();
    let result = f();
    let elapsed = started.elapsed();
    if elapsed.as_millis() >= 50 {
        tracing::debug!(handler = name, elapsed = ?elapsed, "lsp handler");
    } else {
        tracing::trace!(handler = name, elapsed = ?elapsed, "lsp handler");
    }
    result
}

pub(super) fn file_for_uri(workspace: &WorkspaceState, uri: &Url) -> Option<Arc<MagoFile>> {
    let path = uri.to_file_path().ok()?;
    let id = file_id_for(&workspace.root, &path);
    workspace.database.get(&id).ok()
}

fn analyze_all_workspace_files(workspace: &mut WorkspaceState) {
    let host_ids: Vec<FileId> =
        workspace.database.files().filter(|f| matches!(f.file_type, FileType::Host)).map(|f| f.id).collect();
    let count = host_ids.len();
    workspace.refresh_analyses(&host_ids);
    let total_issues: usize = workspace.file_analyses.values().map(|(_, a)| a.lint_issues.len()).sum();
    tracing::info!("mago-server file analysis: {count} files, {total_issues} lint issues");
}
