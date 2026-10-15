//! Keep the client's file watchers in step with the loaded workspaces.

use tokio::sync::watch;
use tokio::task::JoinHandle;
use tower_lsp_server::Client;
use tower_lsp_server::gen_lsp_types::{
    DidChangeWatchedFilesRegistrationOptions, FileSystemWatcher, MessageType, Registration, Unregistration,
};

use super::Backend;

mod patterns;

const REGISTRATION_IDS: [&str; 2] = ["mago-source-files", "mago-source-files-next"];
const METHOD: &str = "workspace/didChangeWatchedFiles";

#[derive(Default)]
struct Registrations {
    patterns: Vec<String>,
    current: Option<&'static str>,
    retired: Option<&'static str>,
}

pub(super) struct Watchers {
    updates: watch::Sender<Vec<String>>,
    task: JoinHandle<()>,
}

impl Watchers {
    pub(super) fn start(client: Client, patterns: Vec<String>) -> Self {
        let (updates, receiver) = watch::channel(patterns);
        let task = tokio::spawn(update_registrations(client, receiver));
        Self { updates, task }
    }

    pub(super) fn update(&self, patterns: Vec<String>) {
        self.updates.send_replace(patterns);
    }

    pub(super) async fn stop(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

impl Drop for Watchers {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Backend {
    pub(super) fn refresh_watchers(&self) {
        if !self.client_preferences().dynamic_watchers {
            return;
        }
        let mut patterns = self
            .with_registry(|registry| {
                registry
                    .iter()
                    .flat_map(|workspace| {
                        let mut patterns = patterns::for_matcher(&workspace.core.matcher);
                        patterns.extend(
                            workspace.core.directory_aliases.targets().flat_map(|path| patterns::for_directory(&path)),
                        );
                        patterns
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        patterns.sort_unstable();
        patterns.dedup();

        if let Some(session) = self.session.get() {
            session.refresh_watchers(&self.client, patterns);
        }
    }
}

async fn update_registrations(client: Client, mut updates: watch::Receiver<Vec<String>>) {
    let mut registered = Registrations::default();
    loop {
        let patterns = updates.borrow_and_update().clone();
        if let Err(error) = registered.replace(&client, &patterns).await {
            client.log_message(MessageType::Error, format!("mago-server: update file watchers: {error}")).await;
        }
        if updates.changed().await.is_err() {
            return;
        }
    }
}

impl Registrations {
    async fn replace(&mut self, client: &Client, patterns: &[String]) -> tower_lsp_server::jsonrpc::Result<()> {
        // Finish failed cleanup before adding another registration; at most two may exist.
        self.remove_retired(client).await?;
        if self.patterns == patterns {
            return Ok(());
        }
        let next = if patterns.is_empty() {
            None
        } else {
            let id = if self.current == Some(REGISTRATION_IDS[0]) { REGISTRATION_IDS[1] } else { REGISTRATION_IDS[0] };
            let watchers = patterns
                .iter()
                .map(|pattern| FileSystemWatcher { glob_pattern: pattern.clone().into(), kind: None })
                .collect();
            // Keep the current watchers until the client accepts their replacement.
            client
                .register_capability(vec![Registration {
                    id: id.into(),
                    method: METHOD.into(),
                    register_options: Some(
                        serde_json::to_value(DidChangeWatchedFilesRegistrationOptions { watchers }).unwrap(),
                    ),
                }])
                .await?;
            Some(id)
        };
        self.retired = std::mem::replace(&mut self.current, next);
        self.patterns = patterns.to_vec();
        self.remove_retired(client).await
    }

    async fn remove_retired(&mut self, client: &Client) -> tower_lsp_server::jsonrpc::Result<()> {
        if let Some(id) = self.retired {
            client.unregister_capability(vec![Unregistration { id: id.into(), method: METHOD.into() }]).await?;
            self.retired = None;
        }
        Ok(())
    }
}
