//! Reload workspaces from scratch once changes to their configuration or ignore rules settle.

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::Weak;
use std::time::Duration;
use std::time::Instant;

use foldhash::HashMap;
use tower_lsp_server::Client;
use tower_lsp_server::gen_lsp_types::MessageType;

use mago_server::workspace::registry::{FileChange, FileEvent};

use crate::language_server::ServerConfig;
use crate::language_server::ordered_service::Tasks;
use crate::language_server::workspace::rebuild_workspace;

use super::Backend;
use super::debounce::Debounced;
use super::session::Session;

const QUIET_PERIOD: Duration = if cfg!(test) { Duration::from_millis(50) } else { Duration::from_secs(1) };

#[derive(Default)]
pub(super) struct Restarts {
    pending: Vec<PendingRestart>,
    scheduled: u64,
}

struct PendingRestart {
    folder: PathBuf,
    cause: PathBuf,
    token: u64,
    held: HashMap<PathBuf, FileChange>,
    timer: Debounced,
}

impl Restarts {
    pub(super) fn cancel(&mut self, folder: &Path) {
        self.pending.retain(|pending| pending.folder != folder);
    }

    pub(super) fn stop(&mut self) -> Vec<Debounced> {
        self.pending.drain(..).map(|pending| pending.timer).collect()
    }

    /// Restart the folder's quiet period with a token no earlier timer or job holds.
    fn trigger(&mut self, folder: PathBuf, cause: PathBuf, timer: impl FnOnce(&Path) -> Debounced) {
        self.scheduled += 1;
        let token = self.scheduled;
        if let Some(pending) = self.pending.iter_mut().find(|pending| pending.folder == folder) {
            pending.token = token;
            pending.timer.schedule(token);
            return;
        }

        tracing::info!(root = %folder.display(), cause = %cause.display(), "workspace restart scheduled");
        let timer = timer(&folder);
        timer.schedule(token);
        self.pending.push(PendingRestart { folder, cause, token, held: HashMap::default(), timer });
    }

    /// A deletion stays held after a later event, so a failed rebuild still drops what was below it.
    fn hold(&mut self, events: &[FileEvent]) {
        for pending in &mut self.pending {
            for event in events {
                let kind = pending.held.entry(event.path.clone()).or_insert(event.kind);
                if *kind != FileChange::Deleted {
                    *kind = event.kind;
                }
            }
        }
    }

    fn take(&mut self, folder: &Path, token: u64) -> Option<PendingRestart> {
        let index = self.pending.iter().position(|pending| pending.folder == folder && pending.token == token)?;
        Some(self.pending.swap_remove(index))
    }
}

/// The restart timers' handle on the backend, which must not keep the session alive.
struct WeakBackend {
    client: Client,
    config: Arc<ServerConfig>,
    session: Weak<OnceLock<Session>>,
    tasks: Tasks,
}

impl WeakBackend {
    fn upgrade(&self) -> Option<Backend> {
        Some(Backend {
            client: self.client.clone(),
            config: Arc::clone(&self.config),
            session: self.session.upgrade()?,
            tasks: self.tasks.clone(),
        })
    }
}

impl Backend {
    /// Schedule restarts for the workspaces these changes trigger. Returns the folders waiting
    /// for a restart: the rebuild reads their changes, which apply only if it fails.
    pub(super) fn hold_for_restarts(&self, events: &[FileEvent]) -> Vec<PathBuf> {
        let Some(session) = self.session.get() else { return Vec::new() };
        let causes = session.with_registry(|registry| registry.restart_causes(events));
        let mut restarts = session.restarts.lock().unwrap();
        if restarts.pending.is_empty() && causes.is_empty() {
            return Vec::new();
        }

        for (folder, cause) in causes {
            restarts.trigger(folder, cause, |folder| self.restart_timer(folder.to_path_buf()));
        }

        restarts.hold(events);

        restarts.pending.iter().map(|pending| pending.folder.clone()).collect()
    }

    fn restart_timer(&self, folder: PathBuf) -> Debounced {
        let backend = WeakBackend {
            client: self.client.clone(),
            config: Arc::clone(&self.config),
            session: Arc::downgrade(&self.session),
            tasks: self.tasks.clone(),
        };

        Debounced::start(QUIET_PERIOD, move |token| {
            if let Some(backend) = backend.upgrade() {
                let folder = folder.clone();
                let tasks = backend.tasks.clone();
                tasks.spawn(async move { backend.restart_workspace(folder, token).await });
            }

            std::future::ready(())
        })
    }

    async fn restart_workspace(&self, folder: PathBuf, token: u64) {
        let Some(session) = self.session.get() else { return };
        let Some(pending) = session.restarts.lock().unwrap().take(&folder, token) else { return };
        let PendingRestart { cause, held, .. } = pending;
        let held: Vec<_> = held.into_iter().map(|(path, kind)| FileEvent { path, kind }).collect();
        if !self.with_registry(|registry| registry.contains_root(&folder)).unwrap_or(false) {
            return;
        }

        let started = Instant::now();
        let failure = match rebuild_workspace(folder.clone(), Arc::clone(&self.config)) {
            Ok(workspace) => {
                let replaced = self.apply_registry_update(move |registry| registry.replace(workspace, held)).await;
                (!replaced).then(|| "analysis failed".to_owned())
            }
            Err(error) => {
                let target = folder.clone();
                self.apply_registry_update(move |registry| registry.change_disk_files(held, |folder| folder == target))
                    .await;
                Some(error)
            }
        };

        match failure {
            None => {
                tracing::info!(root = %folder.display(), elapsed = ?started.elapsed(), "workspace restarted");
                self.client
                    .log_message(
                        MessageType::Info,
                        format!("mago-server: restarted {} after {} changed", folder.display(), cause.display()),
                    )
                    .await;
            }
            Some(error) => {
                tracing::error!(root = %folder.display(), %error, "workspace restart failed");
                self.client
                    .show_message(
                        MessageType::Error,
                        format!(
                            "mago-server: cannot restart {}, keeping its previous state: {error}",
                            folder.display()
                        ),
                    )
                    .await;
            }
        }

        self.refresh_watchers();
        self.notify_capability_change().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stale_restart_jobs_never_take_a_newer_or_readded_restart() {
        let folder = Path::new("/workspace");
        let timer = |_: &Path| Debounced::start(Duration::from_secs(60), |_| std::future::ready(()));
        let mut restarts = Restarts::default();
        restarts.trigger(folder.to_path_buf(), folder.join("mago.toml"), timer);
        restarts.trigger(folder.to_path_buf(), folder.join("mago.toml"), timer);
        assert!(restarts.take(folder, 1).is_none());
        restarts.cancel(folder);
        restarts.trigger(folder.to_path_buf(), folder.join("mago.toml"), timer);
        assert!(restarts.take(folder, 2).is_none());
        assert!(restarts.take(folder, 3).is_some());
    }

    #[tokio::test]
    async fn held_deletions_survive_later_events_for_the_same_path() {
        let folder = Path::new("/workspace");
        let timer = |_: &Path| Debounced::start(Duration::from_secs(60), |_| std::future::ready(()));
        let mut restarts = Restarts::default();
        restarts.trigger(folder.to_path_buf(), folder.join("mago.toml"), timer);
        let package = folder.join("pkg");
        for kind in [FileChange::Deleted, FileChange::Created, FileChange::Changed] {
            restarts.hold(&[FileEvent { path: package.clone(), kind }]);
        }
        let pending = restarts.take(folder, 1).unwrap();
        assert_eq!(pending.held[&package], FileChange::Deleted);
    }
}
