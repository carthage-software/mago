//! Coalesce settled workspace changes without waiting on the client in the ordered worker.

use std::time::Duration;

use tokio::sync::watch;
use tokio::task::JoinHandle;
use tower_lsp_server::Client;

const QUIET_PERIOD: Duration = Duration::from_millis(150);

pub(super) struct CodeLensRefresh {
    updates: watch::Sender<()>,
    task: JoinHandle<()>,
}

impl CodeLensRefresh {
    pub(super) fn start(client: Client) -> Self {
        let (updates, receiver) = watch::channel(());
        Self { updates, task: tokio::spawn(refresh(client, receiver)) }
    }

    pub(super) fn schedule(&self) {
        self.updates.send_replace(());
    }

    pub(super) async fn stop(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

impl Drop for CodeLensRefresh {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn refresh(client: Client, mut updates: watch::Receiver<()>) {
    while updates.changed().await.is_ok() {
        loop {
            match tokio::time::timeout(QUIET_PERIOD, updates.changed()).await {
                Ok(Ok(())) => continue,
                Ok(Err(_)) => return,
                Err(_) => break,
            }
        }
        // Keep at most one request in flight; changes during it share the next refresh.
        if let Err(error) = client.code_lens_refresh().await {
            tracing::debug!(%error, "code-lens refresh failed");
        }
    }
}
