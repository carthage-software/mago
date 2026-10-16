//! Run registry updates on the ordered worker and publish after releasing the state lock.

use std::time::Instant;

use tower_lsp_server::gen_lsp_types::{FileChangeType, FileEvent, MessageType, Uri};

use crate::config::language_server::Provider;
use crate::language_server::registry::{WorkspaceRegistry, WorkspaceUpdate, disk_events};

use super::Backend;

impl Backend {
    /// Database writes and their analysis share one lock, so queries cannot see stale caches.
    /// Returns whether the update finished without errors.
    pub(super) async fn apply_registry_update<F>(&self, mutate: F) -> bool
    where
        F: FnOnce(&mut WorkspaceRegistry) -> WorkspaceUpdate + Send + 'static,
    {
        let started = Instant::now();
        let Some(session) = self.session.get() else { return false };
        let update = session.with_registry_mut(mutate);
        let succeeded = update.errors.is_empty();

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
        succeeded
    }

    pub(super) async fn apply_disk_change(&self, uri: Uri) {
        self.apply_disk_changes(vec![FileEvent { uri, kind: FileChangeType::Changed }]).await;
    }

    pub(super) async fn apply_disk_changes(&self, changes: Vec<FileEvent>) {
        let events = disk_events(changes);
        let held = self.hold_for_restarts(&events);
        self.apply_registry_update(move |registry| {
            registry.change_disk_files(events, |folder| !held.iter().any(|held| held == folder))
        })
        .await;
        self.refresh_watchers();
    }
}
