use std::collections::VecDeque;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
#[cfg(windows)]
use std::net::Ipv4Addr;
#[cfg(windows)]
use std::net::TcpListener;
#[cfg(windows)]
use std::net::TcpStream;
use std::process::Child;
use std::process::ChildStderr;
#[cfg(not(windows))]
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use foldhash::HashMap;
use foldhash::HashMapExt;

use crate::command::WorkerCommand;
use crate::error::WorkerError;
use crate::error::WorkerFailure;
use crate::pool::WorkerPoolOptions;
use crate::protocol::Frame;
use crate::protocol::FrameFlags;
use crate::protocol::FrameKind;

const STATE_RUNNING: u8 = 0;
const STATE_STOPPING: u8 = 1;
const STATE_STOPPED: u8 = 2;
const STATE_FAILED: u8 = 3;
const IO_THREAD_STACK_SIZE: usize = 256 * 1024;
const INITIAL_SHUTDOWN_POLL_INTERVAL: Duration = Duration::from_micros(50);
const MAXIMUM_SHUTDOWN_POLL_INTERVAL: Duration = Duration::from_millis(1);
#[cfg(windows)]
const INPUT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(windows)]
const INPUT_AUTHENTICATION_TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(windows)]
const INPUT_ADDRESS_ENVIRONMENT_VARIABLE: &str = "MAGO_EXTENSION_INPUT_ADDRESS";
#[cfg(windows)]
const INPUT_TOKEN_ENVIRONMENT_VARIABLE: &str = "MAGO_EXTENSION_INPUT_TOKEN";

#[derive(Debug, Default)]
struct WorkerTelemetry {
    frames_sent: AtomicU64,
    frames_received: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    requests: AtomicU64,
    request_ns: AtomicU64,
    successful_responses: AtomicU64,
    remote_errors: AtomicU64,
    nested_requests: AtomicU64,
    nested_handler_ns: AtomicU64,
    notifications: AtomicU64,
    cancellations: AtomicU64,
    timeouts: AtomicU64,
    failures: AtomicU64,
    stderr_bytes: AtomicU64,
    forced_shutdowns: AtomicU64,
}

/// Handles a nested request sent by an extension worker while Mago is waiting
/// for the worker's outer response.
///
/// The successful or error bytes are returned to the worker as an opaque
/// response payload.
pub trait WorkerRequestHandler {
    /// Handles one nested worker request.
    ///
    /// # Errors
    ///
    /// Returns an opaque binary error payload to send back to the worker.
    fn handle(&mut self, request: &Frame) -> Result<Vec<u8>, Vec<u8>>;
}

impl<F> WorkerRequestHandler for F
where
    F: FnMut(&Frame) -> Result<Vec<u8>, Vec<u8>>,
{
    fn handle(&mut self, request: &Frame) -> Result<Vec<u8>, Vec<u8>> {
        self(request)
    }
}

struct AbsentRequestHandler;

impl WorkerRequestHandler for AbsentRequestHandler {
    fn handle(&mut self, _request: &Frame) -> Result<Vec<u8>, Vec<u8>> {
        unreachable!("an absent nested-request handler cannot be invoked")
    }
}

enum WorkerStdin {
    #[cfg(not(windows))]
    Process(ChildStdin),
    #[cfg(windows)]
    Socket(TcpStream),
    #[cfg(test)]
    Test(std::net::TcpStream),
}

impl Write for WorkerStdin {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(not(windows))]
            Self::Process(stdin) => stdin.write(buffer),
            #[cfg(windows)]
            Self::Socket(stream) => stream.write(buffer),
            #[cfg(test)]
            Self::Test(stream) => stream.write(buffer),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(not(windows))]
            Self::Process(stdin) => stdin.flush(),
            #[cfg(windows)]
            Self::Socket(stream) => stream.flush(),
            #[cfg(test)]
            Self::Test(stream) => stream.flush(),
        }
    }
}

enum PendingEvent {
    Frame(Frame),
    Failure(WorkerFailure),
}

struct StderrTail {
    bytes: VecDeque<u8>,
    capacity: usize,
}

impl StderrTail {
    fn new(capacity: usize) -> Self {
        Self { bytes: VecDeque::with_capacity(capacity.min(4096)), capacity }
    }

    fn extend(&mut self, bytes: &[u8]) {
        if self.capacity == 0 {
            return;
        }

        if bytes.len() >= self.capacity {
            self.bytes.clear();
            self.bytes.extend(&bytes[bytes.len() - self.capacity..]);
            return;
        }

        let excess = self.bytes.len().saturating_add(bytes.len()).saturating_sub(self.capacity);
        self.bytes.drain(..excess);
        self.bytes.extend(bytes);
    }

    fn display(&self) -> String {
        let bytes: Vec<_> = self.bytes.iter().copied().collect();
        String::from_utf8_lossy(&bytes).trim().to_owned()
    }
}

struct WorkerInner {
    id: usize,
    state: AtomicU8,
    next_request_id: AtomicU64,
    in_flight: AtomicUsize,
    maximum_payload_size: usize,
    writer: Mutex<Option<BufWriter<WorkerStdin>>>,
    child: Mutex<Option<Child>>,
    pending: Mutex<HashMap<u64, mpsc::Sender<PendingEvent>>>,
    stderr: Mutex<StderrTail>,
    trace_enabled: bool,
    trace_summary_emitted: AtomicBool,
    telemetry: WorkerTelemetry,
    started_at: Option<Instant>,
}

impl WorkerInner {
    fn send(&self, frame: &Frame) -> Result<(), WorkerError> {
        if self.state.load(Ordering::Acquire) != STATE_RUNNING {
            return Err(self.disconnected("worker is not running"));
        }

        let result = {
            let mut writer = lock(&self.writer);
            let writer = writer.as_mut().ok_or_else(|| self.disconnected("worker stdin is closed"))?;
            frame.write_to(writer, self.maximum_payload_size).and_then(|()| writer.flush().map_err(Into::into))
        };

        if let Err(source) = result {
            let error = WorkerError::protocol(self.id, source);
            self.fail(error.to_string());
            return Err(error);
        }

        if self.trace_enabled {
            self.telemetry.frames_sent.fetch_add(1, Ordering::Relaxed);
            self.telemetry
                .bytes_sent
                .fetch_add((crate::protocol::FRAME_HEADER_LENGTH + frame.payload.len()) as u64, Ordering::Relaxed);
        }

        Ok(())
    }

    fn route(&self, frame: Frame) {
        if self.trace_enabled {
            self.telemetry.frames_received.fetch_add(1, Ordering::Relaxed);
            self.telemetry
                .bytes_received
                .fetch_add((crate::protocol::FRAME_HEADER_LENGTH + frame.payload.len()) as u64, Ordering::Relaxed);
        }
        if frame.kind == FrameKind::Shutdown {
            tracing::trace!(worker = self.id, "Extension worker requested host-side shutdown.");
            self.fail("worker requested shutdown");
            return;
        }

        let pending_id = match frame.kind {
            FrameKind::Response | FrameKind::Cancel => frame.id,
            FrameKind::Request | FrameKind::Notification => frame.parent_id,
            FrameKind::Shutdown => return,
        };
        if pending_id == 0 {
            tracing::warn!(worker = self.id, kind = ?frame.kind, "Ignoring unassociated extension worker frame.");
            return;
        }

        let sender = {
            let mut pending = lock(&self.pending);
            if frame.kind == FrameKind::Response {
                pending.remove(&pending_id)
            } else {
                pending.get(&pending_id).cloned()
            }
        };

        if let Some(sender) = sender {
            let _result = sender.send(PendingEvent::Frame(frame));
        } else {
            tracing::debug!(worker = self.id, request = pending_id, "Ignoring late extension worker frame.");
        }
    }

    fn fail(&self, reason: impl Into<String>) {
        if self.state.compare_exchange(STATE_RUNNING, STATE_FAILED, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return;
        }

        if self.trace_enabled {
            self.telemetry.failures.fetch_add(1, Ordering::Relaxed);
        }

        lock(&self.writer).take();
        if let Some(child) = lock(&self.child).as_mut() {
            let _result = child.kill();
        }

        let mut message = reason.into();
        let stderr = lock(&self.stderr).display();
        if !stderr.is_empty() {
            message.push_str("; stderr: ");
            message.push_str(&stderr);
        }

        let failure = WorkerFailure::new(message);
        tracing::trace!(worker = self.id, reason = %failure.message, "Extension worker failed.");
        let pending = std::mem::take(&mut *lock(&self.pending));
        for sender in pending.into_values() {
            let _result = sender.send(PendingEvent::Failure(failure.clone()));
        }
    }

    fn disconnected(&self, message: impl Into<String>) -> WorkerError {
        WorkerError::Disconnected { worker: self.id, message: message.into() }
    }

    fn emit_trace_summary(&self) {
        if !self.trace_enabled || self.trace_summary_emitted.swap(true, Ordering::AcqRel) {
            return;
        }

        let requests = self.telemetry.requests.load(Ordering::Relaxed);
        let request_ns = self.telemetry.request_ns.load(Ordering::Relaxed);
        let nested_requests = self.telemetry.nested_requests.load(Ordering::Relaxed);
        let nested_ns = self.telemetry.nested_handler_ns.load(Ordering::Relaxed);
        tracing::trace!(
            worker = self.id,
            requests,
            successful_responses = self.telemetry.successful_responses.load(Ordering::Relaxed),
            remote_errors = self.telemetry.remote_errors.load(Ordering::Relaxed),
            timeouts = self.telemetry.timeouts.load(Ordering::Relaxed),
            cancellations = self.telemetry.cancellations.load(Ordering::Relaxed),
            total_request_ms = nanos_millis(request_ns),
            average_request_micros = average_micros(request_ns, requests),
            "Extension worker request summary."
        );
        tracing::trace!(
            worker = self.id,
            frames_sent = self.telemetry.frames_sent.load(Ordering::Relaxed),
            frames_received = self.telemetry.frames_received.load(Ordering::Relaxed),
            bytes_sent = self.telemetry.bytes_sent.load(Ordering::Relaxed),
            bytes_received = self.telemetry.bytes_received.load(Ordering::Relaxed),
            notifications = self.telemetry.notifications.load(Ordering::Relaxed),
            "Extension worker protocol summary."
        );
        tracing::trace!(
            worker = self.id,
            nested_requests,
            nested_handler_ms = nanos_millis(nested_ns),
            average_nested_handler_micros = average_micros(nested_ns, nested_requests),
            failures = self.telemetry.failures.load(Ordering::Relaxed),
            stderr_bytes = self.telemetry.stderr_bytes.load(Ordering::Relaxed),
            forced_shutdowns = self.telemetry.forced_shutdowns.load(Ordering::Relaxed),
            lifetime = ?self.started_at.map(|start| start.elapsed()).unwrap_or_default(),
            "Extension worker lifecycle summary."
        );
    }
}

struct WorkerRequestTrace<'worker> {
    inner: &'worker WorkerInner,
    started_at: Instant,
}

impl Drop for WorkerRequestTrace<'_> {
    fn drop(&mut self) {
        self.inner.telemetry.requests.fetch_add(1, Ordering::Relaxed);
        self.inner.telemetry.request_ns.fetch_add(duration_nanos(self.started_at.elapsed()), Ordering::Relaxed);
    }
}

pub(crate) struct Worker {
    inner: Arc<WorkerInner>,
    request_timeout: Duration,
    shutdown_timeout: Duration,
    reader_thread: Mutex<Option<JoinHandle<()>>>,
    stderr_thread: Mutex<Option<JoinHandle<()>>>,
}

impl Worker {
    pub fn spawn(id: usize, command: &WorkerCommand, options: &WorkerPoolOptions) -> Result<Arc<Self>, WorkerError> {
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let started_at = trace_enabled.then(Instant::now);
        tracing::trace!(
            worker = id,
            program = ?command.program(),
            arguments = command.arguments().len(),
            working_directory = ?command.current_directory(),
            "Spawning extension worker process."
        );
        let mut process = command.build();
        #[cfg(windows)]
        let (input_listener, input_address, input_token) = prepare_windows_input()
            .map_err(|source| WorkerError::Spawn { program: command.program().to_owned(), source })?;
        #[cfg(windows)]
        process
            .env(INPUT_ADDRESS_ENVIRONMENT_VARIABLE, input_address)
            .env(INPUT_TOKEN_ENVIRONMENT_VARIABLE, &input_token);

        let mut child =
            process.spawn().map_err(|source| WorkerError::Spawn { program: command.program().to_owned(), source })?;

        let process_stdin = take_pipe(id, "stdin", child.stdin.take(), &mut child)?;
        let stdout = take_pipe(id, "stdout", child.stdout.take(), &mut child)?;
        let stderr = take_pipe(id, "stderr", child.stderr.take(), &mut child)?;
        let process_id = child.id();
        #[cfg(windows)]
        let stdin = match accept_windows_input(&input_listener, &input_token, &mut child) {
            Ok(stream) => {
                drop(process_stdin);
                WorkerStdin::Socket(stream)
            }
            Err(source) => {
                let _result = child.kill();
                let _result = child.wait();
                return Err(WorkerError::Spawn { program: command.program().to_owned(), source });
            }
        };
        #[cfg(not(windows))]
        let stdin = WorkerStdin::Process(process_stdin);

        let inner = Arc::new(WorkerInner {
            id,
            state: AtomicU8::new(STATE_RUNNING),
            next_request_id: AtomicU64::new(1),
            in_flight: AtomicUsize::new(0),
            maximum_payload_size: options.maximum_payload_size,
            writer: Mutex::new(Some(BufWriter::new(stdin))),
            child: Mutex::new(Some(child)),
            pending: Mutex::new(HashMap::new()),
            stderr: Mutex::new(StderrTail::new(options.stderr_tail_size)),
            trace_enabled,
            trace_summary_emitted: AtomicBool::new(false),
            telemetry: WorkerTelemetry::default(),
            started_at,
        });
        tracing::trace!(worker = id, process_id, "Extension worker process spawned.");

        let reader_inner = Arc::clone(&inner);
        let reader_thread = std::thread::Builder::new()
            .name(format!("mago-extension-worker-{id}-stdout"))
            .stack_size(IO_THREAD_STACK_SIZE)
            .spawn(move || read_worker_stdout(&reader_inner, stdout))
            .map_err(|source| {
                inner.fail("failed to start stdout reader thread");
                reap_child(&inner);
                WorkerError::ThreadSpawn { worker: id, role: "stdout reader", source }
            })?;

        let stderr_inner = Arc::clone(&inner);
        let stderr_thread = match std::thread::Builder::new()
            .name(format!("mago-extension-worker-{id}-stderr"))
            .stack_size(IO_THREAD_STACK_SIZE)
            .spawn(move || read_worker_stderr(&stderr_inner, stderr))
        {
            Ok(thread) => thread,
            Err(source) => {
                inner.fail("failed to start stderr reader thread");
                reap_child(&inner);
                let _result = reader_thread.join();
                return Err(WorkerError::ThreadSpawn { worker: id, role: "stderr reader", source });
            }
        };

        let worker = Arc::new(Self {
            inner,
            request_timeout: options.request_timeout,
            shutdown_timeout: options.shutdown_timeout,
            reader_thread: Mutex::new(Some(reader_thread)),
            stderr_thread: Mutex::new(Some(stderr_thread)),
        });
        if let Some(start) = started_at {
            tracing::trace!(worker = id, elapsed = ?start.elapsed(), "Extension worker I/O threads started.");
        }

        Ok(worker)
    }

    pub fn id(&self) -> usize {
        self.inner.id
    }

    pub fn is_running(&self) -> bool {
        self.inner.state.load(Ordering::Acquire) == STATE_RUNNING
    }

    pub fn in_flight(&self) -> usize {
        self.inner.in_flight.load(Ordering::Relaxed)
    }

    pub fn request(&self, payload: Vec<u8>) -> Result<Vec<u8>, WorkerError> {
        self.request_inner::<AbsentRequestHandler>(payload, None)
    }

    pub fn request_with_handler<H>(&self, payload: Vec<u8>, handler: &mut H) -> Result<Vec<u8>, WorkerError>
    where
        H: WorkerRequestHandler,
    {
        self.request_inner(payload, Some(handler))
    }

    fn request_inner<H>(&self, payload: Vec<u8>, mut handler: Option<&mut H>) -> Result<Vec<u8>, WorkerError>
    where
        H: WorkerRequestHandler,
    {
        if !self.is_running() {
            return Err(self.inner.disconnected("worker is not running"));
        }

        let _trace =
            self.inner.trace_enabled.then(|| WorkerRequestTrace { inner: &self.inner, started_at: Instant::now() });
        let request_id = self.next_request_id();
        let (sender, receiver) = mpsc::channel();
        lock(&self.inner.pending).insert(request_id, sender);

        if let Err(error) = self.inner.send(&Frame::request(request_id, payload)) {
            lock(&self.inner.pending).remove(&request_id);
            return Err(error);
        }

        let started_at = Instant::now();
        loop {
            let Some(remaining) = self.request_timeout.checked_sub(started_at.elapsed()) else {
                self.timeout(request_id);
                return Err(WorkerError::Timeout {
                    worker: self.id(),
                    request: request_id,
                    duration: self.request_timeout,
                });
            };

            match receiver.recv_timeout(remaining) {
                Ok(PendingEvent::Frame(frame)) => match frame.kind {
                    FrameKind::Response => {
                        if frame.flags.contains(FrameFlags::ERROR) {
                            if self.inner.trace_enabled {
                                self.inner.telemetry.remote_errors.fetch_add(1, Ordering::Relaxed);
                            }
                            return Err(WorkerError::Remote {
                                worker: self.id(),
                                request: request_id,
                                payload: frame.payload,
                            });
                        }

                        if self.inner.trace_enabled {
                            self.inner.telemetry.successful_responses.fetch_add(1, Ordering::Relaxed);
                        }
                        return Ok(frame.payload);
                    }
                    FrameKind::Request => {
                        if self.inner.trace_enabled {
                            self.inner.telemetry.nested_requests.fetch_add(1, Ordering::Relaxed);
                        }
                        let Some(handler) = handler.as_deref_mut() else {
                            let response = Frame::error(
                                frame.id,
                                request_id,
                                b"Mago did not install a handler for nested worker requests".to_vec(),
                            );
                            let _result = self.inner.send(&response);
                            lock(&self.inner.pending).remove(&request_id);
                            return Err(WorkerError::UnexpectedRequest { worker: self.id(), request: frame.id });
                        };

                        let handler_start = self.inner.trace_enabled.then(Instant::now);
                        let response = match handler.handle(&frame) {
                            Ok(payload) => Frame::response(frame.id, request_id, payload),
                            Err(payload) => Frame::error(frame.id, request_id, payload),
                        };
                        if let Some(start) = handler_start {
                            self.inner
                                .telemetry
                                .nested_handler_ns
                                .fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
                        }
                        self.inner.send(&response)?;
                    }
                    FrameKind::Notification => {
                        if self.inner.trace_enabled {
                            self.inner.telemetry.notifications.fetch_add(1, Ordering::Relaxed);
                        }
                        tracing::trace!(worker = self.id(), request = request_id, "Worker notification received.");
                    }
                    FrameKind::Cancel => {
                        if self.inner.trace_enabled {
                            self.inner.telemetry.cancellations.fetch_add(1, Ordering::Relaxed);
                        }
                        lock(&self.inner.pending).remove(&request_id);
                        return Err(WorkerError::Cancelled { worker: self.id(), request: request_id });
                    }
                    FrameKind::Shutdown => {
                        self.inner.fail("worker requested shutdown");
                    }
                },
                Ok(PendingEvent::Failure(failure)) => {
                    return Err(self.inner.disconnected(failure.message));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    self.timeout(request_id);
                    return Err(WorkerError::Timeout {
                        worker: self.id(),
                        request: request_id,
                        duration: self.request_timeout,
                    });
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    lock(&self.inner.pending).remove(&request_id);
                    return Err(self.inner.disconnected("response channel closed"));
                }
            }
        }
    }

    fn next_request_id(&self) -> u64 {
        loop {
            let id = self.inner.next_request_id.fetch_add(1, Ordering::Relaxed);
            if id != 0 {
                return id;
            }
        }
    }

    fn timeout(&self, request_id: u64) {
        if self.inner.trace_enabled {
            self.inner.telemetry.timeouts.fetch_add(1, Ordering::Relaxed);
        }
        tracing::trace!(worker = self.id(), request = request_id, timeout = ?self.request_timeout, "Extension worker request timed out.");
        lock(&self.inner.pending).remove(&request_id);
        let _result = self.inner.send(&Frame::cancel(request_id));
        self.inner.fail(format!("request {request_id} timed out"));
    }

    pub fn reserve(self: &Arc<Self>) -> WorkerReservation {
        self.inner.in_flight.fetch_add(1, Ordering::Relaxed);
        WorkerReservation { worker: Arc::clone(self) }
    }

    pub fn shutdown(&self) {
        self.begin_shutdown();
        self.finish_shutdown(Instant::now(), self.shutdown_timeout);
    }

    pub fn begin_shutdown(&self) {
        let mut state = self.inner.state.load(Ordering::Acquire);
        loop {
            match state {
                STATE_RUNNING | STATE_FAILED => {
                    match self.inner.state.compare_exchange(state, STATE_STOPPING, Ordering::AcqRel, Ordering::Acquire)
                    {
                        Ok(_) => break,
                        Err(actual) => state = actual,
                    }
                }
                STATE_STOPPING | STATE_STOPPED => return,
                _ => return,
            }
        }

        tracing::trace!(worker = self.id(), previous_state = state, "Beginning extension worker shutdown.");

        {
            let mut writer = lock(&self.inner.writer);
            if state == STATE_RUNNING
                && let Some(mut writer) = writer.take()
            {
                let _result = Frame::shutdown()
                    .write_to(&mut writer, self.inner.maximum_payload_size)
                    .and_then(|()| writer.flush().map_err(Into::into));
            } else {
                writer.take();
            }
        }
    }

    pub fn finish_shutdown(&self, started_at: Instant, timeout: Duration) {
        if self.inner.state.load(Ordering::Acquire) == STATE_STOPPED {
            self.join_threads();
            self.inner.emit_trace_summary();
            return;
        }

        let trace_start = self.inner.trace_enabled.then(Instant::now);
        let mut poll_interval = INITIAL_SHUTDOWN_POLL_INTERVAL;
        let exited_gracefully = loop {
            let exited = {
                let mut child = lock(&self.inner.child);
                match child.as_mut() {
                    Some(child) => child.try_wait().ok().flatten().is_some(),
                    None => true,
                }
            };
            let elapsed = started_at.elapsed();
            if exited || elapsed >= timeout {
                break exited;
            }

            std::thread::sleep(poll_interval.min(timeout.saturating_sub(elapsed)));
            poll_interval = (poll_interval * 2).min(MAXIMUM_SHUTDOWN_POLL_INTERVAL);
        };

        let child = lock(&self.inner.child).take();
        if let Some(mut child) = child {
            if child.try_wait().ok().flatten().is_none() {
                if self.inner.trace_enabled {
                    self.inner.telemetry.forced_shutdowns.fetch_add(1, Ordering::Relaxed);
                }
                tracing::trace!(worker = self.id(), "Killing extension worker after shutdown grace period.");
                let _result = child.kill();
            }
            let _result = child.wait();
        }

        self.inner.state.store(STATE_STOPPED, Ordering::Release);
        let failure = WorkerFailure::new("worker shut down");
        let pending = std::mem::take(&mut *lock(&self.inner.pending));
        for sender in pending.into_values() {
            let _result = sender.send(PendingEvent::Failure(failure.clone()));
        }

        self.join_threads();
        if let Some(start) = trace_start {
            tracing::trace!(
                worker = self.id(),
                graceful = exited_gracefully,
                elapsed = ?start.elapsed(),
                "Extension worker shutdown completed."
            );
        }
        self.inner.emit_trace_summary();
    }

    #[cfg(test)]
    fn from_streams(
        id: usize,
        reader: impl Read + Send + 'static,
        writer: std::net::TcpStream,
        options: &WorkerPoolOptions,
    ) -> Arc<Self> {
        let inner = Arc::new(WorkerInner {
            id,
            state: AtomicU8::new(STATE_RUNNING),
            next_request_id: AtomicU64::new(1),
            in_flight: AtomicUsize::new(0),
            maximum_payload_size: options.maximum_payload_size,
            writer: Mutex::new(Some(BufWriter::new(WorkerStdin::Test(writer)))),
            child: Mutex::new(None),
            pending: Mutex::new(HashMap::new()),
            stderr: Mutex::new(StderrTail::new(options.stderr_tail_size)),
            trace_enabled: tracing::enabled!(tracing::Level::TRACE),
            trace_summary_emitted: AtomicBool::new(false),
            telemetry: WorkerTelemetry::default(),
            started_at: tracing::enabled!(tracing::Level::TRACE).then(Instant::now),
        });
        let reader_inner = Arc::clone(&inner);
        let reader_thread = std::thread::spawn(move || read_worker_stream(&reader_inner, reader));

        Arc::new(Self {
            inner,
            request_timeout: options.request_timeout,
            shutdown_timeout: options.shutdown_timeout,
            reader_thread: Mutex::new(Some(reader_thread)),
            stderr_thread: Mutex::new(None),
        })
    }

    fn join_threads(&self) {
        let reader_thread = lock(&self.reader_thread).take();
        let stderr_thread = lock(&self.stderr_thread).take();
        if reader_thread.is_none() && stderr_thread.is_none() {
            return;
        }

        tracing::trace!(worker = self.id(), "Joining extension worker I/O threads.");
        if let Some(thread) = reader_thread {
            let _result = thread.join();
        }
        if let Some(thread) = stderr_thread {
            let _result = thread.join();
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub(crate) struct WorkerReservation {
    worker: Arc<Worker>,
}

impl WorkerReservation {
    pub fn worker(&self) -> &Worker {
        &self.worker
    }

    pub fn matches(&self, worker: &Arc<Worker>) -> bool {
        Arc::ptr_eq(&self.worker, worker)
    }
}

impl Drop for WorkerReservation {
    fn drop(&mut self) {
        self.worker.inner.in_flight.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(windows)]
fn prepare_windows_input() -> std::io::Result<(TcpListener, String, String)> {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    listener.set_nonblocking(true)?;
    let address = listener.local_addr()?.to_string();
    let random = rand::random::<[u8; 16]>();
    let mut token = String::with_capacity(random.len() * 2);
    for byte in random {
        token.push(char::from(HEX[usize::from(byte >> 4)]));
        token.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }

    Ok((listener, address, token))
}

#[cfg(windows)]
fn accept_windows_input(listener: &TcpListener, token: &str, child: &mut Child) -> std::io::Result<TcpStream> {
    let deadline = Instant::now() + INPUT_CONNECT_TIMEOUT;
    loop {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer.ip().is_loopback() {
                    continue;
                }

                stream.set_read_timeout(Some(INPUT_AUTHENTICATION_TIMEOUT))?;
                let mut received = vec![0; token.len()];
                stream.read_exact(&mut received)?;
                if received != token.as_bytes() {
                    continue;
                }

                stream.set_read_timeout(None)?;
                stream.set_nodelay(true)?;
                tracing::trace!(process = child.id(), %peer, "Extension worker connected its Windows input stream.");
                return Ok(stream);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error),
        }

        if let Some(status) = child.try_wait()? {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("extension worker exited with {status} before connecting its Windows input stream"),
            ));
        }

        if Instant::now() >= deadline {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "extension worker did not connect its Windows input stream",
            ));
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}

fn take_pipe<T>(id: usize, name: &'static str, pipe: Option<T>, child: &mut Child) -> Result<T, WorkerError> {
    pipe.ok_or_else(|| {
        let _result = child.kill();
        let _result = child.wait();
        WorkerError::MissingPipe { worker: id, pipe: name }
    })
}

fn read_worker_stdout(inner: &WorkerInner, stdout: ChildStdout) {
    read_worker_stream(inner, stdout);
}

fn read_worker_stream(inner: &WorkerInner, stream: impl Read) {
    tracing::trace!(worker = inner.id, "Extension worker stdout reader started.");
    let mut reader = BufReader::new(stream);
    loop {
        match Frame::read_from(&mut reader, inner.maximum_payload_size) {
            Ok(Some(frame)) => inner.route(frame),
            Ok(None) => {
                if inner.state.load(Ordering::Acquire) == STATE_RUNNING {
                    inner.fail("worker closed stdout");
                }
                tracing::trace!(worker = inner.id, "Extension worker stdout reader reached EOF.");
                return;
            }
            Err(error) => {
                tracing::trace!(worker = inner.id, error = %error, "Extension worker stdout reader failed.");
                inner.fail(format!("failed to read worker frame: {error}"));
                return;
            }
        }
    }
}

fn read_worker_stderr(inner: &WorkerInner, stderr: ChildStderr) {
    tracing::trace!(worker = inner.id, "Extension worker stderr reader started.");
    let mut reader = BufReader::new(stderr);
    let mut buffer = [0u8; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                tracing::trace!(worker = inner.id, "Extension worker stderr reader reached EOF.");
                return;
            }
            Err(error) => {
                tracing::trace!(worker = inner.id, error = %error, "Extension worker stderr reader failed.");
                return;
            }
            Ok(read) => {
                if inner.trace_enabled {
                    inner.telemetry.stderr_bytes.fetch_add(read as u64, Ordering::Relaxed);
                }
                lock(&inner.stderr).extend(&buffer[..read]);
            }
        }
    }
}

fn reap_child(inner: &WorkerInner) {
    tracing::trace!(worker = inner.id, "Reaping failed extension worker process.");
    let child = lock(&inner.child).take();
    if let Some(mut child) = child {
        let _result = child.kill();
        let _result = child.wait();
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn duration_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[allow(clippy::cast_precision_loss, clippy::float_arithmetic)]
fn nanos_millis(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}

fn average_micros(nanos: u64, count: u64) -> u64 {
    nanos.checked_div(count).unwrap_or(0) / 1_000
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::net::TcpListener;
    use std::net::TcpStream;
    use std::sync::Barrier;

    use super::*;

    #[test]
    fn stderr_tail_keeps_only_the_configured_suffix() {
        let mut tail = StderrTail::new(5);
        tail.extend(b"abc");
        tail.extend(b"defg");

        assert_eq!(tail.display(), "cdefg");
    }

    #[test]
    fn stderr_tail_handles_a_chunk_larger_than_its_capacity() {
        let mut tail = StderrTail::new(4);
        tail.extend(b"abcdefgh");

        assert_eq!(tail.display(), "efgh");
    }

    #[test]
    fn routes_multiplexed_responses_by_request_id() {
        let (worker, mut peer_reader, mut peer_writer) = connected_worker(Duration::from_secs(2));
        let barrier = Arc::new(Barrier::new(3));

        let first_worker = Arc::clone(&worker);
        let first_barrier = Arc::clone(&barrier);
        let first = std::thread::spawn(move || {
            first_barrier.wait();
            first_worker.request(b"first".to_vec()).expect("first request should succeed")
        });
        let second_worker = Arc::clone(&worker);
        let second_barrier = Arc::clone(&barrier);
        let second = std::thread::spawn(move || {
            second_barrier.wait();
            second_worker.request(b"second".to_vec()).expect("second request should succeed")
        });
        barrier.wait();

        let request_a = Frame::read_from(&mut peer_reader, 1024)
            .expect("first request should decode")
            .expect("first request should be present");
        let request_b = Frame::read_from(&mut peer_reader, 1024)
            .expect("second request should decode")
            .expect("second request should be present");
        Frame::response(request_b.id, 0, request_b.payload)
            .write_to(&mut peer_writer, 1024)
            .expect("second response should encode");
        peer_writer.flush().expect("second response should flush");
        Frame::response(request_a.id, 0, request_a.payload)
            .write_to(&mut peer_writer, 1024)
            .expect("first response should encode");
        peer_writer.flush().expect("first response should flush");

        let mut responses =
            [first.join().expect("first thread should join"), second.join().expect("second thread should join")];
        responses.sort();
        assert_eq!(responses, [b"first".to_vec(), b"second".to_vec()]);

        drop(peer_reader);
        drop(peer_writer);
        worker.shutdown();
    }

    #[test]
    fn services_a_nested_worker_request_on_the_calling_thread() {
        let (worker, mut peer_reader, mut peer_writer) = connected_worker(Duration::from_secs(2));
        let peer = std::thread::spawn(move || {
            let outer = Frame::read_from(&mut peer_reader, 1024)
                .expect("outer request should decode")
                .expect("outer request should be present");
            let nested = Frame {
                version: crate::protocol::PROTOCOL_VERSION,
                kind: FrameKind::Request,
                flags: FrameFlags::empty(),
                id: 77,
                parent_id: outer.id,
                payload: b"metadata?".to_vec(),
            };
            nested.write_to(&mut peer_writer, 1024).expect("nested request should encode");
            peer_writer.flush().expect("nested request should flush");

            let nested_response = Frame::read_from(&mut peer_reader, 1024)
                .expect("nested response should decode")
                .expect("nested response should be present");
            assert_eq!(nested_response.kind, FrameKind::Response);
            assert_eq!(nested_response.id, nested.id);
            assert_eq!(nested_response.parent_id, outer.id);
            assert_eq!(nested_response.payload, b"metadata!");

            Frame::response(outer.id, 0, b"done".to_vec())
                .write_to(&mut peer_writer, 1024)
                .expect("outer response should encode");
            peer_writer.flush().expect("outer response should flush");

            let shutdown = Frame::read_from(&mut peer_reader, 1024)
                .expect("shutdown should decode")
                .expect("shutdown should be present");
            assert_eq!(shutdown.kind, FrameKind::Shutdown);
        });

        let mut handler = |request: &Frame| {
            assert_eq!(request.payload, b"metadata?");
            Ok(b"metadata!".to_vec())
        };
        let response =
            worker.request_with_handler(b"analyze".to_vec(), &mut handler).expect("outer request should succeed");
        assert_eq!(response, b"done");

        worker.shutdown();
        peer.join().expect("peer thread should join");
    }

    #[test]
    fn a_timed_out_request_stops_the_worker() {
        let (worker, mut peer_reader, peer_writer) = connected_worker(Duration::from_millis(20));
        let peer = std::thread::spawn(move || {
            let _request = Frame::read_from(&mut peer_reader, 1024)
                .expect("request should decode")
                .expect("request should be present");
            let cancel = Frame::read_from(&mut peer_reader, 1024)
                .expect("cancellation should decode")
                .expect("cancellation should be present");
            assert_eq!(cancel.kind, FrameKind::Cancel);
        });

        let error = worker.request(Vec::new()).expect_err("request should time out");
        assert!(matches!(error, WorkerError::Timeout { .. }));
        assert!(!worker.is_running());

        drop(peer_writer);
        worker.shutdown();
        peer.join().expect("peer thread should join");
    }

    #[test]
    fn reports_worker_cancellation_without_marking_the_worker_disconnected() {
        let (worker, mut peer_reader, mut peer_writer) = connected_worker(Duration::from_secs(2));
        let peer = std::thread::spawn(move || {
            let request = Frame::read_from(&mut peer_reader, 1024)
                .expect("request should decode")
                .expect("request should be present");
            Frame::cancel(request.id).write_to(&mut peer_writer, 1024).expect("cancellation should encode");
            peer_writer.flush().expect("cancellation should flush");

            let shutdown = Frame::read_from(&mut peer_reader, 1024)
                .expect("shutdown should decode")
                .expect("shutdown should be present");
            assert_eq!(shutdown.kind, FrameKind::Shutdown);
        });

        let error = worker.request(Vec::new()).expect_err("request should be cancelled");
        assert!(matches!(error, WorkerError::Cancelled { worker: 0, request: 1 }));
        assert!(worker.is_running());

        worker.shutdown();
        peer.join().expect("peer thread should join");
    }

    fn connected_worker(timeout: Duration) -> (Arc<Worker>, TcpStream, TcpStream) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("listener should bind");
        let address = listener.local_addr().expect("listener should have an address");
        let host_stream = TcpStream::connect(address).expect("host should connect");
        let peer_stream = listener.accept().expect("peer should connect").0;
        let host_reader = host_stream.try_clone().expect("host stream should clone");
        let peer_reader = peer_stream.try_clone().expect("peer stream should clone");
        let options = WorkerPoolOptions {
            maximum_payload_size: 1024,
            request_timeout: timeout,
            shutdown_timeout: Duration::from_millis(100),
            stderr_tail_size: 1024,
        };

        (Worker::from_streams(0, host_reader, host_stream, &options), peer_reader, peer_stream)
    }
}
