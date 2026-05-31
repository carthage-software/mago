//! Helpers attached to [`super::Backend`] that don't belong in the
//! `tower-lsp` trait dispatch: lifecycle (`bootstrap`), workspace mutation
//! (`apply_change_atomic`), and shared utilities.

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::task;
use tower_lsp_server::jsonrpc::Result as JsonRpcResult;
use tower_lsp_server::ls_types::Diagnostic;
use tower_lsp_server::ls_types::MessageType;
use tower_lsp_server::ls_types::Uri;

use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_reporting::CompiledIgnoreSet;
use mago_server::ServerError;
use mago_server::lookup;

use crate::language_server::diagnostics::build_diagnostics;
use crate::language_server::document::OpenDocument;
use crate::language_server::state::BackendState;
use crate::language_server::state::WorkspaceRegistry;
use crate::language_server::state::WorkspaceState;
use crate::language_server::state::build_workspace;
use crate::language_server::workspace::file_id_for;
use crate::language_server::workspace::logical_name_for;

use super::Backend;

impl Backend {
    pub(super) async fn bootstrap(&self, roots: Vec<PathBuf>) {
        let started = Instant::now();
        let mut workspaces = Vec::new();

        for root in roots {
            tracing::info!(root = %root.display(), "bootstrap starting");
            let config = Arc::clone(&self.config);
            let root_label = root.display().to_string();
            let outcome = task::spawn_blocking(move || build_workspace(root, config)).await;

            match outcome {
                Ok(Ok((mut workspace, analysis_result))) => {
                    tracing::info!(
                        root = %root_label,
                        elapsed = ?started.elapsed(),
                        issues = analysis_result.issues.len(),
                        "analyzer pass complete",
                    );

                    if workspace.features.linter {
                        let lint_started = Instant::now();
                        analyze_all_workspace_files(&mut workspace);
                        tracing::info!(elapsed = ?lint_started.elapsed(), "file analysis pass complete");
                    }

                    workspaces.push(workspace);
                }
                Ok(Err(err)) => {
                    tracing::error!(root = %root_label, error = %err, "bootstrap failed");
                    self.client
                        .log_message(
                            MessageType::ERROR,
                            format!("mago-server: bootstrap failed for {root_label}: {err}"),
                        )
                        .await;
                }
                Err(err) => {
                    tracing::error!(root = %root_label, error = %err, "bootstrap task panicked");
                    self.client
                        .log_message(MessageType::ERROR, format!("mago-server: bootstrap task panicked: {err}"))
                        .await;
                }
            }
        }

        let count = workspaces.len();
        *self.state.lock().unwrap() = BackendState::Ready(WorkspaceRegistry::new(workspaces));
        tracing::info!(elapsed = ?started.elapsed(), workspaces = count, "ready");
    }

    /// Apply a database mutation under the workspace mutex AND immediately
    /// run incremental analysis on the affected files; all without
    /// dropping the lock in between.
    ///
    /// This prevents capability handlers (which also acquire the mutex)
    /// from observing a database with new contents but stale analysis
    /// while the analyzer thread is in flight.
    pub(super) async fn apply_change_atomic<F>(&self, uri: Uri, mutate: F)
    where
        F: FnOnce(&mut WorkspaceState) -> Vec<FileId> + Send + 'static,
    {
        let started = Instant::now();
        let state = Arc::clone(&self.state);
        let outcome = task::spawn_blocking(move || -> Result<_, ServerError> {
            let Some(path) = uri.to_file_path() else {
                return Ok(None);
            };

            let mut guard = state.lock().unwrap();
            let BackendState::Ready(registry) = &mut *guard else {
                return Ok(None);
            };
            let Some(workspace) = registry.for_path_mut(path.as_ref()) else {
                return Ok(None);
            };

            let changed = mutate(workspace);
            if changed.is_empty() {
                return Ok(None);
            }

            let mut result = workspace.server.analyze_incremental(&changed)?;

            let ignore_set = CompiledIgnoreSet::compile(
                &workspace.configuration.analyzer.ignore,
                workspace.configuration.source.glob.to_database_settings(),
            );

            result.issues.filter_out_ignored(&ignore_set, |file_id| {
                workspace.database().get_ref(&file_id).ok().map(|f| String::from_utf8_lossy(&f.name).into_owned())
            });

            workspace.invalidate_artifacts(&changed);
            lookup::invalidate(&changed);

            if workspace.features.linter {
                workspace.refresh_analyses(&changed);
            }

            let lint_issues = workspace.server.lint_issues();
            let diagnostics = build_diagnostics(workspace.database(), &result, lint_issues);

            // Diff against this workspace's last-published set so we only emit
            // changed URIs and clear ones that went quiet.
            let stale: Vec<Uri> =
                workspace.last_diagnostics.keys().filter(|url| !diagnostics.contains_key(*url)).cloned().collect();
            let changed_diags: Vec<(Uri, Vec<Diagnostic>)> = diagnostics
                .into_iter()
                .filter(|(url, diags)| workspace.last_diagnostics.get(url) != Some(diags))
                .collect();
            for url in &stale {
                workspace.last_diagnostics.remove(url);
            }
            for (url, diags) in &changed_diags {
                workspace.last_diagnostics.insert(url.clone(), diags.clone());
            }

            Ok(Some((changed.len(), stale, changed_diags)))
        })
        .await;

        match outcome {
            Ok(Ok(Some((count, stale, changed_diags)))) => {
                tracing::debug!(files = count, elapsed = ?started.elapsed(), "incremental analysis");
                for url in stale {
                    self.client.publish_diagnostics(url, vec![], None).await;
                }
                for (url, diags) in changed_diags {
                    self.client.publish_diagnostics(url, diags, None).await;
                }
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

    pub(super) async fn apply_buffer_open(&self, uri: Uri, text: String, version: i32) {
        let Some(path) = uri.to_file_path() else {
            return;
        };
        let path = path.into_owned();
        self.apply_change_atomic(uri.clone(), move |workspace| {
            let logical = logical_name_for(&workspace.root, &path);
            let virtual_file = workspace.database().get_id(logical.as_bytes()).is_none();
            let file_id = if virtual_file {
                let file = MagoFile::new(
                    Cow::Owned(logical.into_bytes()),
                    FileType::Host,
                    Some(path),
                    Cow::Owned(text.into_bytes()),
                );
                workspace.database_mut().add(file)
            } else {
                let id = FileId::new(logical.as_bytes());
                workspace.database_mut().update(id, Cow::Owned(text.into_bytes()));
                id
            };

            workspace.open_documents.insert(uri, OpenDocument { file_id, virtual_file, version });
            vec![file_id]
        })
        .await;
    }

    pub(super) async fn apply_buffer_change(&self, uri: Uri, text: String, version: i32) {
        self.apply_change_atomic(uri.clone(), move |workspace| {
            let file_id = {
                let Some(open) = workspace.open_documents.get_mut(&uri) else {
                    return Vec::new();
                };
                open.version = version;
                open.file_id
            };

            workspace.database_mut().update(file_id, Cow::Owned(text.into_bytes()));
            vec![file_id]
        })
        .await;
    }

    pub(super) async fn apply_buffer_close(&self, uri: Uri) {
        let Some(path) = uri.to_file_path() else {
            return;
        };

        let path = path.into_owned();
        self.apply_change_atomic(uri.clone(), move |workspace| {
            let Some(open) = workspace.open_documents.remove(&uri) else {
                return Vec::new();
            };

            if open.virtual_file {
                workspace.database_mut().delete(open.file_id);
            } else if let Ok(file) = MagoFile::read(&workspace.root, &path, FileType::Host) {
                workspace.database_mut().update(open.file_id, file.contents);
            }
            vec![open.file_id]
        })
        .await;
    }

    pub(super) async fn apply_disk_change(&self, uri: Uri) {
        let Some(path) = uri.to_file_path() else {
            return;
        };
        let path = path.into_owned();
        self.apply_change_atomic(uri.clone(), move |workspace| {
            if workspace.open_documents.contains_key(&uri) {
                return Vec::new();
            }

            let Ok(file) = MagoFile::read(&workspace.root, &path, FileType::Host) else {
                return Vec::new();
            };

            let id = file.id;
            if workspace.database().get(&id).is_ok() {
                workspace.database_mut().update(id, file.contents);
            } else {
                workspace.database_mut().add(file);
            }
            vec![id]
        })
        .await;
    }

    pub(super) async fn apply_disk_delete(&self, uri: Uri) {
        let Some(path) = uri.to_file_path() else {
            return;
        };

        let path = path.into_owned();
        self.apply_change_atomic(uri, move |workspace| {
            let id = file_id_for(&workspace.root, &path);
            if workspace.database_mut().delete(id) { vec![id] } else { Vec::new() }
        })
        .await;
    }

    /// Run `f` with the file at `uri` and its owning workspace, routing by
    /// longest-prefix root match. `None` if uninitialized or unrouted.
    pub(super) fn with_file<F, R>(&self, uri: &Uri, f: F) -> Option<R>
    where
        F: FnOnce(&MagoFile, &WorkspaceState) -> R,
    {
        let path = uri.to_file_path()?;
        let guard = self.state.lock().unwrap();
        let BackendState::Ready(registry) = &*guard else {
            return None;
        };
        let workspace = registry.for_path(path.as_ref())?;
        let file = file_for_uri(workspace, uri)?;
        Some(f(&file, workspace))
    }

    /// Run `f` against the workspace owning `uri`.
    pub(super) fn with_workspace_for_uri<F, R>(&self, uri: &Uri, f: F) -> Option<R>
    where
        F: FnOnce(&WorkspaceState) -> R,
    {
        let path = uri.to_file_path()?;
        let guard = self.state.lock().unwrap();
        let BackendState::Ready(registry) = &*guard else {
            return None;
        };
        let workspace = registry.for_path(path.as_ref())?;
        Some(f(workspace))
    }

    /// Run `f` against the workspace owning `uri`, mutably.
    pub(super) fn with_workspace_mut_for_uri<F, R>(&self, uri: &Uri, f: F) -> Option<R>
    where
        F: FnOnce(&mut WorkspaceState) -> R,
    {
        let path = uri.to_file_path()?;
        let mut guard = self.state.lock().unwrap();
        let BackendState::Ready(registry) = &mut *guard else {
            return None;
        };
        let workspace = registry.for_path_mut(path.as_ref())?;
        Some(f(workspace))
    }

    /// Run `f` against the whole registry (e.g. cross-workspace operations).
    pub(super) fn with_registry<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&WorkspaceRegistry) -> R,
    {
        let guard = self.state.lock().unwrap();
        let BackendState::Ready(registry) = &*guard else {
            return None;
        };
        Some(f(registry))
    }

    /// Run `f` against the whole registry, mutably.
    pub(super) fn with_registry_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut WorkspaceRegistry) -> R,
    {
        let mut guard = self.state.lock().unwrap();
        let BackendState::Ready(registry) = &mut *guard else {
            return None;
        };
        Some(f(registry))
    }

    /// Bootstrap a newly-added workspace folder and insert it into the
    /// registry. No-op if the server isn't ready or the folder is already
    /// tracked.
    pub(super) async fn add_workspace_folder(&self, root: PathBuf) {
        let canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
        if self.with_registry(|registry| registry.contains_root(&canonical)).unwrap_or(true) {
            return;
        }

        let config = Arc::clone(&self.config);
        let root_label = root.display().to_string();
        let outcome = task::spawn_blocking(move || build_workspace(root, config)).await;

        match outcome {
            Ok(Ok((mut workspace, _analysis_result))) => {
                if workspace.features.linter {
                    analyze_all_workspace_files(&mut workspace);
                }
                self.with_registry_mut(|registry| registry.add(workspace));
                tracing::info!(root = %root_label, "workspace folder added");
            }
            Ok(Err(err)) => {
                tracing::error!(root = %root_label, error = %err, "adding workspace folder failed");
            }
            Err(err) => {
                tracing::error!(root = %root_label, error = %err, "add-workspace task panicked");
            }
        }
    }

    /// Remove a workspace folder from the registry and clear the diagnostics it
    /// had published.
    pub(super) async fn remove_workspace_folder(&self, root: PathBuf) {
        let canonical = root.canonicalize().unwrap_or(root);
        let stale: Vec<Uri> = self
            .with_registry_mut(|registry| {
                registry.remove(&canonical).map(|ws| ws.last_diagnostics.into_keys().collect()).unwrap_or_default()
            })
            .unwrap_or_default();

        for uri in stale {
            self.client.publish_diagnostics(uri, vec![], None).await;
        }
    }
}

/// Run a synchronous LSP capability handler with a tracing span attached
/// for telemetry. Slow handlers (≥50ms) log at `debug`; the rest at `trace`.
pub(super) fn traced<T, F>(name: &'static str, f: F) -> JsonRpcResult<T>
where
    F: FnOnce() -> JsonRpcResult<T>,
{
    let started = Instant::now();
    let result = f();
    let elapsed = started.elapsed();
    if elapsed.as_millis() >= 50 {
        tracing::debug!(handler = name, elapsed = ?elapsed, "lsp handler");
    } else {
        tracing::trace!(handler = name, elapsed = ?elapsed, "lsp handler");
    }
    result
}

pub(super) fn file_for_uri(workspace: &WorkspaceState, uri: &Uri) -> Option<Arc<MagoFile>> {
    let path = uri.to_file_path()?;
    let id = file_id_for(&workspace.root, &path);
    workspace.database().get(&id).ok()
}

fn analyze_all_workspace_files(workspace: &mut WorkspaceState) {
    workspace.server.refresh_all_host_analyses();
    let count = workspace.server.analyses().count();
    let total_issues: usize = workspace.server.lint_issues().map(|issues| issues.len()).sum();
    tracing::info!("mago-server file analysis: {count} files, {total_issues} lint issues");
}
