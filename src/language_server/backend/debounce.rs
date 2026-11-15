//! Coalesce settled workspace changes without waiting on the client in the ordered worker.

use std::future::Future;
use std::time::Duration;

use tokio::sync::watch;
use tokio::task::JoinHandle;

pub(super) struct Debounced {
    updates: watch::Sender<u64>,
    task: JoinHandle<()>,
}

impl Debounced {
    /// Run `action` with the last scheduled token once none was scheduled for `quiet`.
    pub(super) fn start<A, F>(quiet: Duration, mut action: A) -> Self
    where
        A: FnMut(u64) -> F + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        let (updates, mut receiver) = watch::channel(0);
        let task = tokio::spawn(async move {
            while receiver.changed().await.is_ok() {
                let mut token = *receiver.borrow_and_update();
                loop {
                    match tokio::time::timeout(quiet, receiver.changed()).await {
                        Ok(Ok(())) => token = *receiver.borrow_and_update(),
                        Ok(Err(_)) => return,
                        Err(_) => break,
                    }
                }
                // Keep at most one action in flight; changes during it share the next run.
                action(token).await;
            }
        });

        Self { updates, task }
    }

    pub(super) fn schedule(&self, token: u64) {
        self.updates.send_replace(token);
    }

    pub(super) async fn stop(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

impl Drop for Debounced {
    fn drop(&mut self) {
        self.task.abort();
    }
}
