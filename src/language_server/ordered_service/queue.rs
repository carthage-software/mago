//! Run queued handlers in order and track the owner of each pending request ID.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::future::Future;
use std::io;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::Weak;

use tokio::runtime::Handle;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tower_lsp_server::jsonrpc::Error;
use tower_lsp_server::jsonrpc::Id;
use tower_lsp_server::jsonrpc::Request;
use tower_lsp_server::jsonrpc::Response;

use super::Handler;
use super::Output;
use super::SessionState;

struct Job {
    handler: Handler,
    reply: oneshot::Sender<Output>,
}

pub(super) struct Slot {
    id: Option<Id>,
    job: Mutex<Option<Job>>,
}

impl Slot {
    /// Returns the request unchanged if the worker already took this slot.
    pub(super) fn replace_queued(
        &self,
        request: Request,
        prepare: impl FnOnce(Request) -> io::Result<Handler>,
    ) -> io::Result<Option<Request>> {
        let mut queued = self.job.lock().unwrap();
        let Some(Job { handler, reply }) = queued.take() else {
            return Ok(Some(request));
        };
        // Free the old text before reserving its replacement, keeping the slot and reply.
        drop(handler);
        *queued = Some(Job { handler: prepare(request)?, reply });
        Ok(None)
    }
}

#[derive(Clone, Default)]
struct PendingRequests(Arc<Mutex<HashMap<Id, Weak<Slot>>>>);

impl PendingRequests {
    fn find(&self, id: &Id) -> Option<Arc<Slot>> {
        self.0.lock().unwrap().get(id).and_then(Weak::upgrade)
    }

    fn insert(&self, slot: &Arc<Slot>) {
        if let Some(id) = &slot.id {
            // Tower rejects duplicate IDs; keep the first request as the cancellation target.
            self.0.lock().unwrap().entry(id.clone()).or_insert_with(|| Arc::downgrade(slot));
        }
    }

    fn remove(&self, slot: &Arc<Slot>) {
        if let Some(id) = &slot.id {
            let mut pending = self.0.lock().unwrap();
            if pending.get(id).is_some_and(|entry| entry.ptr_eq(&Arc::downgrade(slot))) {
                pending.remove(id);
            }
        }
    }
}

#[derive(Default)]
struct Waiting {
    jobs: VecDeque<Arc<Slot>>,
    requests: usize,
    pushed: u64,
    closed: bool,
}

struct WorkerQueue {
    waiting: Mutex<Waiting>,
    ready: Condvar,
    capacity: usize,
}

impl WorkerQueue {
    fn push(&self, slot: Arc<Slot>) -> io::Result<u64> {
        let mut waiting = self.waiting.lock().unwrap();
        if waiting.closed {
            return Err(io::Error::other("language server worker stopped"));
        }
        if slot.id.is_some() {
            if waiting.requests >= self.capacity {
                return Err(io::Error::other("language server request queue is full"));
            }
            waiting.requests += 1;
        }
        waiting.jobs.push_back(slot);
        waiting.pushed += 1;
        let pushed = waiting.pushed;
        drop(waiting);
        self.ready.notify_one();
        Ok(pushed)
    }

    fn pop(&self) -> Option<Arc<Slot>> {
        let mut waiting = self.waiting.lock().unwrap();
        loop {
            if let Some(slot) = waiting.jobs.pop_front() {
                if slot.id.is_some() {
                    waiting.requests -= 1;
                }
                return Some(slot);
            }
            if waiting.closed {
                return None;
            }
            waiting = self.ready.wait(waiting).unwrap();
        }
    }

    fn remove(&self, slot: &Arc<Slot>) {
        let mut waiting = self.waiting.lock().unwrap();
        let queued = waiting.jobs.len();
        waiting.jobs.retain(|queued| !Arc::ptr_eq(queued, slot));
        if waiting.jobs.len() < queued && slot.id.is_some() {
            waiting.requests -= 1;
        }
    }

    fn close(&self) {
        let mut waiting = self.waiting.lock().unwrap();
        waiting.closed = true;
        waiting.requests = 0;
        let jobs = std::mem::take(&mut waiting.jobs);
        drop(waiting);
        self.ready.notify_all();
        drop(jobs);
    }
}

pub(super) struct Queue {
    jobs: Arc<WorkerQueue>,
    pending: PendingRequests,
    session: Weak<Mutex<SessionState>>,
}

/// Queue the server's own work on the worker, behind the messages already waiting.
/// A task runs only while the session is running.
#[derive(Clone, Default)]
pub(in crate::language_server) struct Tasks {
    jobs: Weak<WorkerQueue>,
    session: Weak<Mutex<SessionState>>,
}

impl Tasks {
    pub(in crate::language_server) fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
        let Some(jobs) = self.jobs.upgrade() else { return };
        let session = self.session.clone();
        let (reply, _) = oneshot::channel();
        let handler: Handler = Box::pin(async move {
            if session.upgrade().is_some_and(|session| matches!(*session.lock().unwrap(), SessionState::Running)) {
                task.await;
            }
            Ok(None)
        });
        let _ = jobs.push(Arc::new(Slot { id: None, job: Mutex::new(Some(Job { handler, reply })) }));
    }
}

impl Queue {
    pub(super) fn new(
        capacity: usize,
        stopped: watch::Sender<bool>,
        session: Arc<Mutex<SessionState>>,
    ) -> io::Result<Self> {
        let jobs = Arc::new(WorkerQueue { waiting: Mutex::default(), ready: Condvar::new(), capacity });
        let tasks_session = Arc::downgrade(&session);
        let worker_jobs = Arc::clone(&jobs);
        let pending = PendingRequests::default();
        let worker_pending = pending.clone();
        let mut stop = stopped.subscribe();
        let runtime = Handle::current();
        std::thread::Builder::new().name("mago-lsp".into()).spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                while let Some(slot) = worker_jobs.pop() {
                    if *stop.borrow() {
                        break;
                    }
                    let Some(job) = slot.job.lock().unwrap().take() else { continue };
                    // Poll admitted handlers even if their replies drop; Tower clears IDs on completion.
                    let result = runtime.block_on(async {
                        tokio::select! {
                            biased;
                            _ = stop.changed() => None,
                            result = job.handler => Some(result),
                        }
                    });
                    worker_pending.remove(&slot);
                    let Some(result) = result else { break };
                    if let Err(error) = &result {
                        *session.lock().unwrap() = SessionState::Failed(error.to_string());
                        let _ = job.reply.send(result);
                        break;
                    }
                    let _ = job.reply.send(result);
                }
            }));

            if outcome.is_err() {
                *session.lock().unwrap() = SessionState::Failed("language server worker panicked".into());
                tracing::error!("language server worker panicked");
            }

            stopped.send_replace(true);
            worker_jobs.close();
        })?;

        Ok(Self { jobs, pending, session: tasks_session })
    }

    pub(super) fn find(&self, id: &Id) -> Option<Arc<Slot>> {
        self.pending.find(id)
    }

    pub(super) fn tasks(&self) -> Tasks {
        Tasks { jobs: Arc::downgrade(&self.jobs), session: self.session.clone() }
    }

    /// Replace a queued notification only if nothing was queued after the push numbered `pushed`.
    pub(super) fn replace_queued(
        &self,
        slot: &Slot,
        pushed: u64,
        request: Request,
        prepare: impl FnOnce(Request) -> io::Result<Handler>,
    ) -> io::Result<Option<Request>> {
        let waiting = self.jobs.waiting.lock().unwrap();
        if waiting.pushed != pushed {
            return Ok(Some(request));
        }
        let replaced = slot.replace_queued(request, prepare);
        drop(waiting);
        replaced
    }

    pub(super) fn is_full(&self) -> bool {
        self.jobs.waiting.lock().unwrap().requests >= self.jobs.capacity
    }

    pub(super) fn cancel(&self, handler: Handler, cancelled: Option<Arc<Slot>>) -> Handler {
        // Poll cancellation even when Tower's ordinary handler slots are full.
        let pending = self.pending.clone();
        let jobs = Arc::clone(&self.jobs);
        let task = tokio::spawn(async move {
            let result = handler.await;
            if let Some(slot) = cancelled.filter(|_| matches!(&result, Ok(None))) {
                let job = slot.job.lock().unwrap().take();
                if let Some(Job { handler, reply }) = job {
                    // Tower has removed the ID. Drop its old future before the client can reuse it.
                    drop(handler);
                    pending.remove(&slot);
                    // Release capacity before the client sees the reply and reuses the request ID.
                    jobs.remove(&slot);
                    let response = Response::from_error(slot.id.clone().unwrap(), Error::request_cancelled());
                    let _ = reply.send(Ok(Some(response)));
                }
            }
            result
        });
        Box::pin(async move { task.await.map_err(io::Error::other)? })
    }

    pub(super) fn enqueue(
        &self,
        id: Option<Id>,
        handler: Handler,
        mut stop: watch::Receiver<bool>,
    ) -> io::Result<(Weak<Slot>, Handler, u64)> {
        let (reply, response) = oneshot::channel();
        let notification = id.is_none();
        let slot = Arc::new(Slot { id, job: Mutex::new(Some(Job { handler, reply })) });
        self.pending.insert(&slot);
        let pushed = match self.jobs.push(Arc::clone(&slot)) {
            Ok(pushed) => pushed,
            Err(error) => {
                self.pending.remove(&slot);
                return Err(error);
            }
        };

        if notification {
            return Ok((Arc::downgrade(&slot), Box::pin(std::future::ready(Ok(None))), pushed));
        }

        let reply = Box::pin(async move {
            if *stop.borrow() {
                return Ok(None);
            }
            tokio::select! {
                result = response => result.map_err(|_| io::Error::other("language server worker stopped"))?,
                _ = stop.changed() => Ok(None),
            }
        });

        Ok((Arc::downgrade(&slot), reply, pushed))
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        self.jobs.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(id: i64) -> Arc<Slot> {
        Arc::new(Slot { id: Some(id.into()), job: Mutex::new(None) })
    }

    #[tokio::test(flavor = "current_thread")]
    async fn dropping_an_empty_queue_stops_its_worker() {
        let (stopped, mut stop) = watch::channel(false);
        let queue = Queue::new(1, stopped, Arc::new(Mutex::new(SessionState::default()))).unwrap();
        drop(queue);
        tokio::time::timeout(std::time::Duration::from_secs(5), stop.changed()).await.unwrap().unwrap();
        assert!(*stop.borrow());
    }

    #[test]
    fn duplicate_slots_cannot_replace_or_remove_the_first_request() {
        let pending = PendingRequests::default();
        let first = slot(1);
        let duplicate = slot(1);
        pending.insert(&first);
        pending.insert(&duplicate);
        assert!(Arc::ptr_eq(&pending.find(&1.into()).unwrap(), &first));
        pending.remove(&duplicate);
        assert!(Arc::ptr_eq(&pending.find(&1.into()).unwrap(), &first));
        pending.remove(&first);
        assert!(pending.find(&1.into()).is_none());
    }

    #[test]
    fn stale_slot_cleanup_cannot_remove_a_reused_request_id() {
        let pending = PendingRequests::default();
        let old = slot(1);
        pending.insert(&old);
        pending.remove(&old);
        let current = slot(1);
        pending.insert(&current);
        pending.remove(&old);
        assert!(Arc::ptr_eq(&pending.find(&1.into()).unwrap(), &current));
        pending.remove(&current);
        assert!(pending.find(&1.into()).is_none());
    }

    #[test]
    fn a_started_slot_returns_the_request_without_preparing_a_handler() {
        let slot = slot(1);
        let request = Request::build("textDocument/didChange").finish();
        let returned = slot.replace_queued(request.clone(), |_| panic!("handler must not be prepared"));
        assert_eq!(returned.unwrap(), Some(request));
    }
}
