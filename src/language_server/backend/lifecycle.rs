//! Load and update workspaces on the ordered worker.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tower_lsp_server::gen_lsp_types::MessageType;

use crate::language_server::registry::{WorkspaceRegistry, WorkspaceUpdate};
use crate::language_server::workspace::build_workspace;

use super::Backend;

impl Backend {
    pub(super) async fn bootstrap(&self, roots: Vec<PathBuf>) -> WorkspaceRegistry {
        let started = Instant::now();
        let mut workspaces = Vec::new();

        for root in roots {
            // Let cancellation stop a multi-root load before starting the next workspace.
            tokio::task::yield_now().await;
            tracing::info!(root = %root.display(), "bootstrap starting");
            let config = Arc::clone(&self.config);
            let root_label = root.display().to_string();

            match build_workspace(root, config) {
                Ok(workspace) => workspaces.push(workspace),
                Err(err) => {
                    tracing::error!(root = %root_label, error = %err, "bootstrap failed");
                    self.client
                        .log_message(
                            MessageType::Error,
                            format!("mago-server: bootstrap failed for {root_label}: {err}"),
                        )
                        .await;
                }
            }
        }

        let count = workspaces.len();
        let registry = WorkspaceRegistry::new(workspaces);
        tracing::info!(elapsed = ?started.elapsed(), workspaces = count, "ready");
        registry
    }

    /// Bootstrap a newly-added workspace folder and insert it into the
    /// registry. No-op if the server isn't ready or the folder is already
    /// tracked.
    pub(super) async fn add_workspace_folder(&self, root: PathBuf) {
        let canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
        if self
            .with_registry(|registry| registry.contains_root(&root) || registry.contains_root(&canonical))
            .unwrap_or(true)
        {
            return;
        }

        let config = Arc::clone(&self.config);
        let root_label = root.display().to_string();

        match build_workspace(root, config) {
            Ok(workspace) => {
                self.apply_registry_update(move |registry| registry.add(workspace)).await;
            }
            Err(err) => {
                tracing::error!(root = %root_label, error = %err, "adding workspace folder failed");
            }
        }
    }

    /// Remove a folder and publish diagnostics from the remaining owners.
    pub(super) async fn remove_workspace_folder(&self, root: PathBuf) {
        if let Some(session) = self.session.get() {
            session.restarts.lock().unwrap().cancel(&root);
        }
        self.apply_registry_update(move |registry| {
            let refresh_code_lenses = registry.contains_root(&root);
            WorkspaceUpdate { diagnostics: registry.remove(&root), refresh_code_lenses, ..WorkspaceUpdate::default() }
        })
        .await;
    }
}
