//! Run registry updates on the ordered worker and publish after releasing the state lock.

use std::time::Instant;

use tower_lsp_server::gen_lsp_types::{FileChangeType, FileEvent, MessageType, Uri};

use crate::config::language_server::Provider;
use crate::language_server::registry::{WorkspaceRegistry, WorkspaceUpdate};

use super::Backend;

impl Backend {
    /// Database writes and their analysis share one lock, so queries cannot see stale caches.
    pub(super) async fn apply_registry_update<F>(&self, mutate: F)
    where
        F: FnOnce(&mut WorkspaceRegistry) -> WorkspaceUpdate + Send + 'static,
    {
        let started = Instant::now();
        let Some(session) = self.session.get() else { return };
        let update = session.with_registry_mut(mutate);

        if update.changed_files > 0 {
            tracing::debug!(files = update.changed_files, elapsed = ?started.elapsed(), "incremental analysis");
        }

        for error in update.errors {
            tracing::error!(%error, "incremental analysis failed");
            self.client
                .log_message(MessageType::Error, format!("mago-server: incremental analysis failed: {error}"))
                .await;
        }

        self.publish_diagnostics(update.diagnostics).await;
        if update.refresh_code_lenses
            && self.config.supports_provider(Provider::CodeLens)
            && self.providers().is_enabled(Provider::CodeLens)
        {
            session.refresh_code_lenses(&self.client);
        }
    }

    pub(super) async fn apply_disk_change(&self, uri: Uri) {
        self.apply_disk_changes(vec![FileEvent { uri, kind: FileChangeType::Changed }]).await;
    }

    pub(super) async fn apply_disk_changes(&self, changes: Vec<FileEvent>) {
        self.apply_registry_update(move |registry| registry.change_disk_files(changes)).await;
        self.refresh_watchers();
    }
}
