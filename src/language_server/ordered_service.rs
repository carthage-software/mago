//! Run handlers in wire order on one worker, leaving the transport free to poll I/O.

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;
use std::task::Context;
use std::task::Poll;

use serde::Deserialize;
use tokio::sync::watch;
use tower_lsp_server::gen_lsp_types::CancelParams;
use tower_lsp_server::gen_lsp_types::Uri;
use tower_lsp_server::jsonrpc::Id;
use tower_lsp_server::jsonrpc::Request;
use tower_lsp_server::jsonrpc::Response;
use tower_service::Service;

use self::input::CONTROL_BYTES_LIMIT;
use self::input::INPUT_BYTES_LIMIT;
use self::input::InputBudget;
use self::input::encoded_size;
use self::lifecycle::Admission;
use self::lifecycle::Initialization;
use self::lifecycle::SessionState;
use self::queue::Queue;
use self::queue::Slot;

mod input;
mod lifecycle;
mod queue;

const QUEUE_CAPACITY: usize = 128;
// One active handler plus every queued handler leaves one slot for busy replies and controls.
// Refused requests must stay ready so a full worker queue cannot fill all reply slots.
pub(super) const REPLY_CONCURRENCY: usize = QUEUE_CAPACITY + 2;

type Output = io::Result<Option<Response>>;
type Handler = Pin<Box<dyn Future<Output = Output> + Send>>;

fn full_change(request: &Request) -> Option<(Uri, i32)> {
    if request.id().is_some() || request.method() != "textDocument/didChange" {
        return None;
    }
    let params = request.params()?;
    let document = params.get("textDocument")?;
    let version = i32::try_from(document.get("version")?.as_i64()?).ok()?;
    let changes = params.get("contentChanges")?.as_array()?;
    let [change] = changes.as_slice() else { return None };
    change.get("text")?.as_str()?;
    if ["range", "rangeLength"].iter().any(|field| change.get(field).is_some_and(|value| !value.is_null())) {
        return None;
    }
    Some((Uri::deserialize(document.get("uri")?).ok()?, version))
}

pub(super) struct OrderedService<S> {
    inner: S,
    queue: Queue,
    last_change: Option<(Uri, i32, Weak<Slot>)>,
    input: InputBudget,
    stopped: watch::Sender<bool>,
    session: Arc<Mutex<SessionState>>,
}

impl<S> OrderedService<S> {
    pub(super) fn new(inner: S) -> io::Result<Self> {
        Self::with_capacity(inner, QUEUE_CAPACITY)
    }

    fn with_capacity(inner: S, capacity: usize) -> io::Result<Self> {
        let (stopped, _) = watch::channel(false);
        let session = Arc::new(Mutex::new(SessionState::default()));
        let queue = Queue::new(capacity, stopped.clone(), Arc::clone(&session))?;
        Ok(Self { inner, queue, stopped, session, last_change: None, input: InputBudget::default() })
    }

    pub(super) fn stopped(&self) -> watch::Receiver<bool> {
        self.stopped.subscribe()
    }

    pub(super) fn session(&self) -> Arc<Mutex<SessionState>> {
        Arc::clone(&self.session)
    }

    fn fail(&self, error: io::Error) -> Handler {
        tracing::error!(%error, "language server stopped");
        *self.session.lock().unwrap() = SessionState::Failed(error.to_string());
        self.stopped.send_replace(true);
        Box::pin(std::future::ready(Err(error)))
    }
}

impl<S> OrderedService<S>
where
    S: Service<Request, Response = Option<Response>>,
    S::Future: Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    fn prepare_handler(&mut self, request: Request, bytes: usize, control: bool) -> io::Result<Handler> {
        let permit = self.input.reserve(bytes, control)?;
        let initialize = (request.method() == "initialize").then(|| Initialization::start(Arc::clone(&self.session)));
        // Reserve before tower converts the payload and registers the request ID.
        let handler = self.inner.call(request);
        Ok(Box::pin(async move {
            let _permit = permit;
            let result = handler.await.map_err(io::Error::other);
            if let Some(initialize) = initialize {
                initialize.complete(&result);
            }
            result
        }))
    }
}

impl<S> Drop for OrderedService<S> {
    fn drop(&mut self) {
        // Never join here: a running synchronous handler cannot stop until it returns.
        self.stopped.send_replace(true);
    }
}

impl<S> Service<Request> for OrderedService<S>
where
    S: Service<Request, Response = Option<Response>>,
    S::Future: Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = Option<Response>;
    type Error = io::Error;
    type Future = Handler;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        if *self.stopped.borrow() {
            return Poll::Ready(Err(io::Error::other("language server worker stopped")));
        }
        self.inner.poll_ready(context).map_err(io::Error::other)
    }

    fn call(&mut self, request: Request) -> Handler {
        let exit = request.method() == "exit";
        let cancel = request.method() == "$/cancelRequest";
        let bytes = match encoded_size(&request, if exit || cancel { CONTROL_BYTES_LIMIT } else { INPUT_BYTES_LIMIT }) {
            Ok(bytes) => bytes,
            Err(error) => return self.fail(error),
        };
        let change = full_change(&request);
        let previous = self.last_change.take();
        let request = if let Some((uri, version)) = &change
            && let Some((previous_uri, previous_version, previous_slot)) = previous
            && *uri == previous_uri
            && *version > previous_version
            && let Some(slot) = previous_slot.upgrade()
        {
            match slot.replace_queued(request, |request| self.prepare_handler(request, bytes, false)) {
                Ok(None) => {
                    self.last_change = Some((uri.clone(), *version, Arc::downgrade(&slot)));
                    return Box::pin(std::future::ready(Ok(None)));
                }
                Ok(Some(request)) => request,
                Err(error) => return self.fail(error),
            }
        } else {
            request
        };
        let id = request.id().cloned();
        let cancelled = if cancel && id.is_none() {
            request
                .params()
                .and_then(|params| CancelParams::deserialize(params).ok())
                .and_then(|params| self.queue.find(&Id::from(params.id)))
        } else {
            None
        };
        // This is the only enqueuer; the worker and cancellation can only free capacity.
        // Check before Tower registers the ID or shutdown changes the session state.
        let overloaded = id.is_some() && !exit && !cancel && (self.queue.is_full() || !self.input.has_capacity(bytes));
        if overloaded || matches!(request.method(), "initialize" | "shutdown" | "exit") {
            let duplicate_id = id.as_ref().is_some_and(|id| self.queue.find(id).is_some());
            match self.session.lock().unwrap().admit(&request, duplicate_id, overloaded) {
                Admission::Forward => {}
                Admission::Reply(response) => return Box::pin(std::future::ready(Ok(response))),
            }
        }
        // Tower must register request IDs and check lifecycle state at arrival.
        let handler = match self.prepare_handler(request, bytes, exit || cancel) {
            Ok(handler) => handler,
            Err(error) => return self.fail(error),
        };
        if exit {
            self.stopped.send_replace(true);
            return handler;
        }
        if cancel {
            return self.queue.cancel(handler, cancelled);
        }

        let (slot, reply) = match self.queue.enqueue(id, handler, self.stopped()) {
            Ok(queued) => queued,
            // Stop rather than lose a document change and serve stale text.
            Err(error) => return self.fail(error),
        };
        self.last_change = change.map(|(uri, version)| (uri, version, slot));
        reply
    }
}

#[cfg(test)]
mod tests;
