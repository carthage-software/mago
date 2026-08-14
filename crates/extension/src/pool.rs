use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use crate::command::WorkerCommand;
use crate::error::WorkerError;
use crate::reduction;
use crate::worker::Worker;
use crate::worker::WorkerRequestHandler;

const INITIAL_WORKERS: usize = 2;
const WARMUP_REQUESTS: usize = 8;
const GROWTH_SAMPLE_MULTIPLIER: usize = 2;
const GROWTH_REQUEST_NANOS: u64 = 1_000_000;
const SLOW_REQUEST_THRESHOLD: Duration = Duration::from_millis(10);
const STATE_RUNNING: u8 = 0;
const STATE_FINALIZING: u8 = 1;
const STATE_STOPPING: u8 = 2;
const STATE_STOPPED: u8 = 3;

#[derive(Debug, Default)]
struct PoolTelemetry {
    requests: AtomicU64,
    request_errors: AtomicU64,
    request_bytes: AtomicU64,
    response_bytes: AtomicU64,
    request_ns: AtomicU64,
    reserve_ns: AtomicU64,
    exchange_ns: AtomicU64,
    recovery_ns: AtomicU64,
    contended_reservations: AtomicU64,
    peak_in_flight: AtomicUsize,
    broadcasts: AtomicU64,
    broadcast_workers: AtomicU64,
    broadcast_ns: AtomicU64,
    restarts: AtomicU64,
    restart_ns: AtomicU64,
    growths: AtomicU64,
    growth_ns: AtomicU64,
    bootstrap_replays: AtomicU64,
    bootstrap_ns: AtomicU64,
}

/// Runtime and safety limits shared by every process in a worker pool.
#[derive(Debug, Clone)]
pub struct WorkerPoolOptions {
    /// Maximum payload bytes accepted in one protocol frame.
    pub maximum_payload_size: usize,
    /// Deadline for one outer request, including nested worker requests.
    pub request_timeout: Duration,
    /// Grace period between the shutdown frame and forcibly killing a worker.
    pub shutdown_timeout: Duration,
    /// Number of trailing stderr bytes retained for failure diagnostics.
    pub stderr_tail_size: usize,
}

impl Default for WorkerPoolOptions {
    fn default() -> Self {
        Self {
            maximum_payload_size: 64 * 1024 * 1024,
            request_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_millis(250),
            stderr_tail_size: 64 * 1024,
        }
    }
}

struct WorkerSlot {
    worker: Mutex<Option<Arc<Worker>>>,
    restart: Mutex<()>,
}

#[derive(Debug)]
struct Bootstrap {
    request: Vec<u8>,
    response: Vec<u8>,
}

/// A fixed or adaptive pool of persistent, multiplexed extension processes.
///
/// Scheduling prefers the live worker with the fewest requests in flight.
/// An adaptive pool starts small to avoid multiplying language-runtime startup
/// costs, then grows toward its configured ceiling when completed requests
/// prove the extension workload is CPU-heavy. Each process may cooperatively
/// interleave multiple requests when its runtime supports that.
pub struct WorkerPool {
    command: WorkerCommand,
    options: WorkerPoolOptions,
    workers: Box<[WorkerSlot]>,
    active_workers: AtomicUsize,
    growth: Mutex<()>,
    bootstraps: Mutex<Vec<Bootstrap>>,
    completed_requests: AtomicUsize,
    request_nanos: AtomicU64,
    cursor: AtomicUsize,
    state: AtomicU8,
    worker_reduction: AtomicBool,
    trace_enabled: bool,
    telemetry: PoolTelemetry,
    started_at: Option<Instant>,
}

impl std::fmt::Debug for WorkerPool {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (bootstrap_count, bootstrap_request_bytes, bootstrap_response_bytes) = {
            let bootstraps = lock(&self.bootstraps);
            (
                bootstraps.len(),
                bootstraps.iter().map(|bootstrap| bootstrap.request.len()).sum::<usize>(),
                bootstraps.iter().map(|bootstrap| bootstrap.response.len()).sum::<usize>(),
            )
        };

        formatter
            .debug_struct("WorkerPool")
            .field("command", &self.command)
            .field("options", &self.options)
            .field("worker_count", &self.active_workers.load(Ordering::Relaxed))
            .field("worker_capacity", &self.workers.len())
            .field("growth", &self.growth)
            .field("bootstrap_count", &bootstrap_count)
            .field("bootstrap_request_bytes", &bootstrap_request_bytes)
            .field("bootstrap_response_bytes", &bootstrap_response_bytes)
            .field("completed_requests", &self.completed_requests.load(Ordering::Relaxed))
            .field("request_nanos", &self.request_nanos.load(Ordering::Relaxed))
            .field("cursor", &self.cursor.load(Ordering::Relaxed))
            .field("state", &pool_state_name(self.state.load(Ordering::Relaxed)))
            .field("worker_reduction", &self.worker_reduction.load(Ordering::Relaxed))
            .field("trace_enabled", &self.trace_enabled)
            .field("telemetry", &self.telemetry)
            .field("started_at", &self.started_at)
            .finish()
    }
}

impl WorkerPool {
    /// Starts a fixed pool of `size` identical worker processes.
    ///
    /// # Errors
    ///
    /// Returns an error if a worker process or one of its stream reader threads
    /// cannot be started.
    pub fn spawn(command: WorkerCommand, size: NonZeroUsize, options: WorkerPoolOptions) -> Result<Self, WorkerError> {
        Self::spawn_with_initial_size(command, size, size.get(), options)
    }

    /// Creates a pool that starts small and may grow to `size` workers.
    ///
    /// # Errors
    ///
    /// Returns an error if an eager worker process or one of its stream reader
    /// threads cannot be started.
    pub fn spawn_adaptive(
        command: WorkerCommand,
        size: NonZeroUsize,
        options: WorkerPoolOptions,
    ) -> Result<Self, WorkerError> {
        Self::spawn_with_initial_size(command, size, size.get().min(INITIAL_WORKERS), options)
    }

    fn spawn_with_initial_size(
        command: WorkerCommand,
        size: NonZeroUsize,
        initial_workers: usize,
        options: WorkerPoolOptions,
    ) -> Result<Self, WorkerError> {
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let started_at = trace_enabled.then(Instant::now);
        tracing::trace!(
            program = ?command.program(),
            configured_workers = size.get(),
            initial_workers,
            adaptive = initial_workers < size.get(),
            maximum_payload_bytes = options.maximum_payload_size,
            request_timeout_ms = options.request_timeout.as_millis(),
            shutdown_timeout_ms = options.shutdown_timeout.as_millis(),
            stderr_tail_bytes = options.stderr_tail_size,
            "Starting extension worker pool."
        );
        let mut workers = Vec::with_capacity(size.get());
        for id in 0..size.get() {
            let worker = if id < initial_workers { Some(Worker::spawn(id, &command, &options)?) } else { None };
            workers.push(WorkerSlot { worker: Mutex::new(worker), restart: Mutex::new(()) });
        }

        if let Some(start) = started_at {
            tracing::trace!(
                program = ?command.program(),
                active_workers = initial_workers,
                worker_capacity = size.get(),
                elapsed = ?start.elapsed(),
                "Extension worker pool started."
            );
        }

        Ok(Self {
            command,
            options,
            workers: workers.into_boxed_slice(),
            active_workers: AtomicUsize::new(initial_workers),
            growth: Mutex::new(()),
            bootstraps: Mutex::new(Vec::new()),
            completed_requests: AtomicUsize::new(0),
            request_nanos: AtomicU64::new(0),
            cursor: AtomicUsize::new(0),
            state: AtomicU8::new(STATE_RUNNING),
            worker_reduction: AtomicBool::new(false),
            trace_enabled,
            telemetry: PoolTelemetry::default(),
            started_at,
        })
    }

    /// Number of processes currently active in this pool.
    #[must_use]
    pub fn len(&self) -> usize {
        if self.state.load(Ordering::Acquire) == STATE_RUNNING {
            self.active_workers.load(Ordering::Acquire)
        } else {
            0
        }
    }

    /// Returns whether this pool has been shut down and can no longer serve requests.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.state.load(Ordering::Acquire) != STATE_RUNNING
    }

    /// Enables terminal worker-state reduction for this pool.
    ///
    /// Capability registration calls this only when at least one logical
    /// extension advertises a worker reducer. Keeping it opt-in means ordinary
    /// worker pools pay no shutdown request or payload cost.
    pub fn enable_worker_reduction(&self) {
        self.worker_reduction.store(true, Ordering::Release);
        tracing::trace!(active_workers = self.active_workers.load(Ordering::Relaxed), "Enabled worker reduction.");
    }

    /// Sends a request through the least-loaded worker and blocks for its response.
    ///
    /// # Errors
    ///
    /// Returns an error when no worker is available, communication fails, the
    /// deadline expires, or the worker returns an error response.
    pub fn request(&self, payload: Vec<u8>) -> Result<Vec<u8>, WorkerError> {
        let start = Instant::now();
        let trace_start = self.trace_enabled.then(Instant::now);
        let request_bytes = payload.len();
        let reserve_start = self.trace_enabled.then(Instant::now);
        let (index, reservation) = match self.reserve_worker() {
            Ok(reservation) => reservation,
            Err(error) => {
                self.record_trace_request(
                    request_bytes,
                    0,
                    trace_start.map_or(Duration::ZERO, |start| start.elapsed()),
                    reserve_start.map_or(Duration::ZERO, |start| start.elapsed()),
                    Duration::ZERO,
                    Duration::ZERO,
                    true,
                    false,
                );
                return Err(error);
            }
        };
        let reserve_duration = reserve_start.map_or(Duration::ZERO, |start| start.elapsed());
        let exchange_start = self.trace_enabled.then(Instant::now);
        let result = reservation.worker().request(payload);
        let exchange_duration = exchange_start.map_or(Duration::ZERO, |start| start.elapsed());
        let recovery_start = self.trace_enabled.then(Instant::now);
        let recovery = self.recover_if_needed(index, &reservation, &result);
        let recovery_duration = recovery_start.map_or(Duration::ZERO, |start| start.elapsed());
        let response_bytes = result.as_ref().map_or(0, Vec::len);
        let failed = result.is_err() || recovery.is_err();
        self.record_trace_request(
            request_bytes,
            response_bytes,
            trace_start.map_or(Duration::ZERO, |start| start.elapsed()),
            reserve_duration,
            exchange_duration,
            recovery_duration,
            failed,
            false,
        );
        recovery?;
        self.record_request(start.elapsed());
        result
    }

    /// Sends a request and services nested worker requests on the calling thread.
    ///
    /// # Errors
    ///
    /// Returns an error when no worker is available, communication fails, the
    /// deadline expires, the nested handler fails to respond, or the worker
    /// returns an error response.
    pub fn request_with_handler<H>(&self, payload: Vec<u8>, handler: &mut H) -> Result<Vec<u8>, WorkerError>
    where
        H: WorkerRequestHandler,
    {
        let start = Instant::now();
        let trace_start = self.trace_enabled.then(Instant::now);
        let request_bytes = payload.len();
        let reserve_start = self.trace_enabled.then(Instant::now);
        let (index, reservation) = match self.reserve_worker() {
            Ok(reservation) => reservation,
            Err(error) => {
                self.record_trace_request(
                    request_bytes,
                    0,
                    trace_start.map_or(Duration::ZERO, |start| start.elapsed()),
                    reserve_start.map_or(Duration::ZERO, |start| start.elapsed()),
                    Duration::ZERO,
                    Duration::ZERO,
                    true,
                    true,
                );
                return Err(error);
            }
        };
        let reserve_duration = reserve_start.map_or(Duration::ZERO, |start| start.elapsed());
        let exchange_start = self.trace_enabled.then(Instant::now);
        let result = reservation.worker().request_with_handler(payload, handler);
        let exchange_duration = exchange_start.map_or(Duration::ZERO, |start| start.elapsed());
        let recovery_start = self.trace_enabled.then(Instant::now);
        let recovery = self.recover_if_needed(index, &reservation, &result);
        let recovery_duration = recovery_start.map_or(Duration::ZERO, |start| start.elapsed());
        let response_bytes = result.as_ref().map_or(0, Vec::len);
        let failed = result.is_err() || recovery.is_err();
        self.record_trace_request(
            request_bytes,
            response_bytes,
            trace_start.map_or(Duration::ZERO, |start| start.elapsed()),
            reserve_duration,
            exchange_duration,
            recovery_duration,
            failed,
            true,
        );
        recovery?;
        self.record_request(start.elapsed());
        result
    }

    /// Sends the same request to every active worker concurrently and returns
    /// results in worker-index order.
    ///
    /// This is intended for initialization and registration validation, where
    /// every process in a pool must begin with identical extension state. The
    /// exchange is recorded and replayed by workers started later.
    ///
    /// # Errors
    ///
    /// Returns an error if any worker cannot be started, communicated with, or
    /// restarted after failure. All coordinator threads are joined before the
    /// error is returned.
    pub fn broadcast(&self, payload: &[u8]) -> Result<Vec<Vec<u8>>, WorkerError> {
        let trace_start = self.trace_enabled.then(Instant::now);
        if self.state.load(Ordering::Acquire) != STATE_RUNNING {
            tracing::trace!(payload_bytes = payload.len(), "Rejected extension broadcast while pool is shutting down.");
            return Err(WorkerError::Unavailable);
        }

        let _initialization = lock(&self.growth);
        if self.state.load(Ordering::Acquire) != STATE_RUNNING {
            tracing::trace!(payload_bytes = payload.len(), "Rejected extension broadcast during pool finalization.");
            return Err(WorkerError::Unavailable);
        }
        let active_workers = self.active_workers.load(Ordering::Acquire);
        tracing::trace!(
            workers = active_workers,
            payload_bytes = payload.len(),
            "Broadcasting extension initialization request."
        );
        let workers = (0..active_workers)
            .map(|index| self.ensure_running(index).map(|worker| (index, worker)))
            .collect::<Result<Vec<_>, _>>()?;

        let results = std::thread::scope(|scope| {
            workers
                .into_iter()
                .map(|(index, worker)| {
                    (
                        index,
                        scope.spawn(move || {
                            let reservation = worker.reserve();
                            let result = reservation.worker().request(payload.to_vec());
                            (reservation, result)
                        }),
                    )
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(index, handle)| (index, handle.join()))
                .collect::<Vec<_>>()
        });

        let mut responses = vec![None; active_workers];
        let mut first_error = None;
        for (index, result) in results {
            let Ok((reservation, result)) = result else {
                first_error.get_or_insert(WorkerError::CoordinatorPanic { worker: index });
                continue;
            };

            if let Err(error) = self.recover_if_needed(index, &reservation, &result) {
                first_error.get_or_insert(error);
            }

            match result {
                Ok(response) => responses[index] = Some(response),
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }

        if let Some(error) = first_error {
            if let Some(start) = trace_start {
                self.telemetry.broadcasts.fetch_add(1, Ordering::Relaxed);
                self.telemetry.broadcast_workers.fetch_add(active_workers as u64, Ordering::Relaxed);
                self.telemetry.broadcast_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
                tracing::trace!(
                    workers = active_workers,
                    payload_bytes = payload.len(),
                    elapsed = ?start.elapsed(),
                    error = %error,
                    "Extension initialization broadcast failed."
                );
            }
            return Err(error);
        }

        let responses = responses.into_iter().flatten().collect::<Vec<_>>();
        let response_bytes = responses.iter().map(Vec::len).sum::<usize>();
        if let Some(response) = responses.first() {
            lock(&self.bootstraps).push(Bootstrap { request: payload.to_vec(), response: response.clone() });
        }

        if let Some(start) = trace_start {
            let elapsed = start.elapsed();
            self.telemetry.broadcasts.fetch_add(1, Ordering::Relaxed);
            self.telemetry.broadcast_workers.fetch_add(active_workers as u64, Ordering::Relaxed);
            self.telemetry.broadcast_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);
            tracing::trace!(
                workers = active_workers,
                request_bytes = payload.len(),
                response_bytes,
                elapsed = ?elapsed,
                "Extension initialization broadcast completed."
            );
        }

        Ok(responses)
    }

    /// Gracefully stops every worker, then kills workers that exceed the
    /// configured shutdown grace period.
    pub fn shutdown(&self) {
        if self.state.compare_exchange(STATE_RUNNING, STATE_FINALIZING, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return;
        }

        if tracing::enabled!(tracing::Level::TRACE) {
            let completed = self.completed_requests.load(Ordering::Relaxed);
            let measured = completed.saturating_sub(WARMUP_REQUESTS);
            let request_nanos = self.request_nanos.load(Ordering::Relaxed);
            let traced_requests = self.telemetry.requests.load(Ordering::Relaxed);
            tracing::trace!(
                completed_requests = completed,
                active_workers = self.active_workers.load(Ordering::Relaxed),
                average_request_micros = if measured == 0 { 0 } else { request_nanos / measured as u64 / 1_000 },
                "Shutting down extension worker pool."
            );
            tracing::trace!(
                requests = traced_requests,
                errors = self.telemetry.request_errors.load(Ordering::Relaxed),
                request_bytes = self.telemetry.request_bytes.load(Ordering::Relaxed),
                response_bytes = self.telemetry.response_bytes.load(Ordering::Relaxed),
                total_request_ms = nanos_millis(self.telemetry.request_ns.load(Ordering::Relaxed)),
                average_request_micros =
                    average_micros(self.telemetry.request_ns.load(Ordering::Relaxed), traced_requests),
                "Extension worker pool request summary."
            );
            tracing::trace!(
                reserve_ms = nanos_millis(self.telemetry.reserve_ns.load(Ordering::Relaxed)),
                exchange_ms = nanos_millis(self.telemetry.exchange_ns.load(Ordering::Relaxed)),
                recovery_ms = nanos_millis(self.telemetry.recovery_ns.load(Ordering::Relaxed)),
                contended_reservations = self.telemetry.contended_reservations.load(Ordering::Relaxed),
                peak_in_flight = self.telemetry.peak_in_flight.load(Ordering::Relaxed),
                "Extension worker pool scheduling and IPC summary."
            );
            tracing::trace!(
                broadcasts = self.telemetry.broadcasts.load(Ordering::Relaxed),
                broadcast_workers = self.telemetry.broadcast_workers.load(Ordering::Relaxed),
                broadcast_ms = nanos_millis(self.telemetry.broadcast_ns.load(Ordering::Relaxed)),
                restarts = self.telemetry.restarts.load(Ordering::Relaxed),
                restart_ms = nanos_millis(self.telemetry.restart_ns.load(Ordering::Relaxed)),
                growths = self.telemetry.growths.load(Ordering::Relaxed),
                growth_ms = nanos_millis(self.telemetry.growth_ns.load(Ordering::Relaxed)),
                bootstrap_replays = self.telemetry.bootstrap_replays.load(Ordering::Relaxed),
                bootstrap_ms = nanos_millis(self.telemetry.bootstrap_ns.load(Ordering::Relaxed)),
                lifetime = ?self.started_at.map(|start| start.elapsed()).unwrap_or_default(),
                "Extension worker pool lifecycle summary."
            );
        }

        let _lifecycle = lock(&self.growth);
        if self.worker_reduction.load(Ordering::Acquire)
            && let Err(error) = self.reduce_worker_state()
        {
            tracing::warn!(error = %error, "Extension worker reduction failed; continuing pool shutdown.");
        }

        self.state.store(STATE_STOPPING, Ordering::Release);
        let workers =
            self.workers.iter().filter_map(|slot| lock(&slot.worker).as_ref().map(Arc::clone)).collect::<Vec<_>>();
        self.shutdown_workers(&workers);
        self.state.store(STATE_STOPPED, Ordering::Release);

        tracing::trace!("Extension worker pool shut down.");
    }

    fn reduce_worker_state(&self) -> Result<(), WorkerError> {
        let started_at = Instant::now();
        let active_workers = self.active_workers.load(Ordering::Acquire);
        let workers = (0..active_workers)
            .map(|index| {
                let worker = lock(&self.workers[index].worker).as_ref().map(Arc::clone).ok_or_else(|| {
                    WorkerError::Reduction { worker: index, message: "active worker slot is empty".to_string() }
                })?;
                if !worker.is_running() {
                    return Err(WorkerError::Reduction {
                        worker: index,
                        message: "worker stopped before its state could be collected".to_string(),
                    });
                }

                Ok(worker)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let drain_started_at = Instant::now();
        while workers.iter().any(|worker| worker.in_flight() != 0) {
            std::thread::sleep(Duration::from_micros(100));
        }
        tracing::trace!(
            workers = workers.len(),
            elapsed = ?drain_started_at.elapsed(),
            "Drained extension requests before worker reduction."
        );

        let request = reduction::collect_request();
        tracing::trace!(
            workers = workers.len(),
            request_bytes = request.len(),
            "Collecting process-local extension state from workers."
        );
        let results = std::thread::scope(|scope| {
            workers
                .iter()
                .enumerate()
                .map(|(index, worker)| {
                    let request = request.clone();
                    (index, scope.spawn(move || worker.request(request)))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(index, handle)| (index, handle.join()))
                .collect::<Vec<_>>()
        });

        let mut responses = vec![None; workers.len()];
        let mut reducers = None;
        let mut first_error = None;
        for (index, result) in results {
            let Ok(result) = result else {
                first_error.get_or_insert(WorkerError::CoordinatorPanic { worker: index });
                continue;
            };

            match result {
                Ok(response) => match reduction::decode_collect_response(index, &response) {
                    Ok(worker_has_reducers) => {
                        if reducers.is_some_and(|reducers| reducers != worker_has_reducers) {
                            first_error.get_or_insert_with(|| WorkerError::Reduction {
                                worker: index,
                                message: "workers advertised inconsistent reducer registrations".to_string(),
                            });
                        } else {
                            reducers = Some(worker_has_reducers);
                            responses[index] = Some(response);
                        }
                    }
                    Err(error) => {
                        first_error.get_or_insert(error);
                    }
                },
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }

        if let Some(error) = first_error {
            return Err(error);
        }

        if reducers != Some(true) {
            tracing::trace!(workers = workers.len(), "No worker reducer data was collected.");
            return Ok(());
        }

        let responses = responses.into_iter().flatten().collect::<Vec<_>>();
        let leader = &workers[0];
        let leader_index = leader.id();
        let followers = &workers[1..];
        let request = reduction::reduce_request(leader_index, &responses, self.options.maximum_payload_size)?;
        let response_bytes = responses.iter().map(Vec::len).sum::<usize>();
        tracing::trace!(
            leader = leader_index,
            followers = followers.len(),
            "Stopping follower extension workers before reduction."
        );
        self.shutdown_workers(followers);

        tracing::trace!(
            leader = leader_index,
            workers = responses.len(),
            collected_bytes = response_bytes,
            request_bytes = request.len(),
            "Sending the complete worker reduction batch to the surviving worker."
        );
        let response = leader.request(request)?;
        reduction::decode_reduce_response(leader_index, &response)?;
        tracing::trace!(
            leader = leader_index,
            workers = responses.len(),
            collected_bytes = response_bytes,
            elapsed = ?started_at.elapsed(),
            "Extension worker reduction completed."
        );

        Ok(())
    }

    fn shutdown_workers(&self, workers: &[Arc<Worker>]) {
        for worker in workers {
            worker.begin_shutdown();
        }

        let started_at = Instant::now();
        for worker in workers {
            worker.finish_shutdown(started_at, self.options.shutdown_timeout);
        }
    }

    fn reserve_worker(&self) -> Result<(usize, crate::worker::WorkerReservation), WorkerError> {
        if self.state.load(Ordering::Acquire) != STATE_RUNNING {
            return Err(WorkerError::Unavailable);
        }

        let active_workers = self.active_workers.load(Ordering::Acquire);
        let start = self.cursor.fetch_add(1, Ordering::Relaxed) % active_workers;
        let mut selected: Option<(usize, Arc<Worker>, usize)> = None;
        let mut last_error = None;

        for offset in 0..active_workers {
            let index = (start + offset) % active_workers;
            let worker = match self.ensure_running(index) {
                Ok(worker) => worker,
                Err(error) => {
                    last_error = Some(error);
                    continue;
                }
            };
            let load = worker.in_flight();
            if selected.as_ref().is_none_or(|(_, _, selected_load)| load < *selected_load) {
                selected = Some((index, worker, load));
            }
        }

        let Some((index, worker, load)) = selected else {
            tracing::trace!(active_workers, "No running extension worker could be reserved.");
            return Err(last_error.unwrap_or(WorkerError::Unavailable));
        };

        if self.trace_enabled {
            if load > 0 {
                self.telemetry.contended_reservations.fetch_add(1, Ordering::Relaxed);
            }
            self.telemetry.peak_in_flight.fetch_max(load.saturating_add(1), Ordering::Relaxed);
        }

        if load > 0
            && self.should_grow(active_workers)
            && let Some((index, worker)) = self.try_grow(active_workers)?
        {
            let reservation = worker.reserve();
            if self.state.load(Ordering::Acquire) != STATE_RUNNING {
                return Err(WorkerError::Unavailable);
            }

            return Ok((index, reservation));
        }

        let reservation = worker.reserve();
        if self.state.load(Ordering::Acquire) != STATE_RUNNING {
            return Err(WorkerError::Unavailable);
        }

        Ok((index, reservation))
    }

    fn ensure_running(&self, index: usize) -> Result<Arc<Worker>, WorkerError> {
        let worker = lock(&self.workers[index].worker).as_ref().map(Arc::clone).ok_or(WorkerError::Unavailable)?;
        if worker.is_running() {
            return Ok(worker);
        }

        tracing::trace!(worker = index, "Extension worker is not running; attempting restart.");
        self.restart(index, &worker)
    }

    fn recover_if_needed(
        &self,
        index: usize,
        reservation: &crate::worker::WorkerReservation,
        result: &Result<Vec<u8>, WorkerError>,
    ) -> Result<(), WorkerError> {
        if result.is_ok() || reservation.worker().is_running() || self.state.load(Ordering::Acquire) != STATE_RUNNING {
            return Ok(());
        }

        let failed = lock(&self.workers[index].worker).as_ref().map(Arc::clone);
        if let Some(failed) = failed
            && reservation.matches(&failed)
        {
            self.restart(index, &failed)?;
        }

        Ok(())
    }

    fn restart(&self, index: usize, failed: &Arc<Worker>) -> Result<Arc<Worker>, WorkerError> {
        let trace_start = self.trace_enabled.then(Instant::now);
        tracing::trace!(worker = index, "Restarting extension worker.");
        let _restart = lock(&self.workers[index].restart);
        let mut slot = lock(&self.workers[index].worker);
        if let Some(current) = slot.as_ref()
            && !Arc::ptr_eq(current, failed)
            && current.is_running()
        {
            return Ok(Arc::clone(current));
        }

        failed.shutdown();
        let replacement = self
            .spawn_initialized(index)
            .map_err(|source| WorkerError::Restart { worker: index, source: Box::new(source) })?;
        *slot = Some(Arc::clone(&replacement));

        if let Some(start) = trace_start {
            let elapsed = start.elapsed();
            self.telemetry.restarts.fetch_add(1, Ordering::Relaxed);
            self.telemetry.restart_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);
            tracing::trace!(worker = index, elapsed = ?elapsed, "Extension worker restarted.");
        }

        Ok(replacement)
    }

    fn try_grow(&self, observed_workers: usize) -> Result<Option<(usize, Arc<Worker>)>, WorkerError> {
        if observed_workers >= self.workers.len() {
            return Ok(None);
        }

        let Ok(_growth) = self.growth.try_lock() else {
            return Ok(None);
        };
        if self.state.load(Ordering::Acquire) != STATE_RUNNING {
            return Ok(None);
        }
        let active_workers = self.active_workers.load(Ordering::Acquire);
        if active_workers != observed_workers
            || active_workers >= self.workers.len()
            || !self.should_grow(active_workers)
        {
            return Ok(None);
        }

        let trace_start = self.trace_enabled.then(Instant::now);
        tracing::trace!(
            active_workers,
            worker_capacity = self.workers.len(),
            "Growing adaptive extension worker pool."
        );
        let worker = self.spawn_initialized(active_workers)?;
        *lock(&self.workers[active_workers].worker) = Some(Arc::clone(&worker));
        self.active_workers.store(active_workers + 1, Ordering::Release);
        if let Some(start) = trace_start {
            let elapsed = start.elapsed();
            self.telemetry.growths.fetch_add(1, Ordering::Relaxed);
            self.telemetry.growth_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);
            tracing::trace!(workers = active_workers + 1, elapsed = ?elapsed, "Expanded adaptive extension worker pool.");
        }

        Ok(Some((active_workers, worker)))
    }

    fn spawn_initialized(&self, index: usize) -> Result<Arc<Worker>, WorkerError> {
        let trace_start = self.trace_enabled.then(Instant::now);
        let worker = Worker::spawn(index, &self.command, &self.options)?;
        let bootstraps = lock(&self.bootstraps);
        tracing::trace!(worker = index, bootstrap_requests = bootstraps.len(), "Initializing extension worker state.");
        for bootstrap in bootstraps.iter() {
            let bootstrap_start = self.trace_enabled.then(Instant::now);
            let response = match worker.request(bootstrap.request.clone()) {
                Ok(response) => response,
                Err(error) => {
                    worker.shutdown();
                    return Err(error);
                }
            };
            if response != bootstrap.response {
                tracing::trace!(worker = index, "Extension worker returned inconsistent bootstrap metadata.");
                worker.shutdown();
                return Err(WorkerError::InconsistentInitialization { worker: index });
            }
            if let Some(start) = bootstrap_start {
                self.telemetry.bootstrap_replays.fetch_add(1, Ordering::Relaxed);
                self.telemetry.bootstrap_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            }
        }

        if let Some(start) = trace_start {
            tracing::trace!(
                worker = index,
                bootstrap_requests = bootstraps.len(),
                elapsed = ?start.elapsed(),
                "Extension worker state initialized."
            );
        }

        Ok(worker)
    }

    fn should_grow(&self, active_workers: usize) -> bool {
        let completed = self.completed_requests.load(Ordering::Relaxed);
        let samples = completed.saturating_sub(WARMUP_REQUESTS);
        if samples < active_workers * GROWTH_SAMPLE_MULTIPLIER {
            return false;
        }

        self.request_nanos.load(Ordering::Relaxed) / samples as u64 >= GROWTH_REQUEST_NANOS
    }

    fn record_request(&self, duration: Duration) {
        let completed = self.completed_requests.fetch_add(1, Ordering::Relaxed);
        if completed >= WARMUP_REQUESTS {
            let nanos = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
            self.request_nanos.fetch_add(nanos, Ordering::Relaxed);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn record_trace_request(
        &self,
        request_bytes: usize,
        response_bytes: usize,
        total: Duration,
        reserve: Duration,
        exchange: Duration,
        recovery: Duration,
        failed: bool,
        nested_handler: bool,
    ) {
        if !self.trace_enabled {
            return;
        }

        self.telemetry.requests.fetch_add(1, Ordering::Relaxed);
        self.telemetry.request_errors.fetch_add(u64::from(failed), Ordering::Relaxed);
        self.telemetry.request_bytes.fetch_add(request_bytes as u64, Ordering::Relaxed);
        self.telemetry.response_bytes.fetch_add(response_bytes as u64, Ordering::Relaxed);
        self.telemetry.request_ns.fetch_add(duration_nanos(total), Ordering::Relaxed);
        self.telemetry.reserve_ns.fetch_add(duration_nanos(reserve), Ordering::Relaxed);
        self.telemetry.exchange_ns.fetch_add(duration_nanos(exchange), Ordering::Relaxed);
        self.telemetry.recovery_ns.fetch_add(duration_nanos(recovery), Ordering::Relaxed);

        if total >= SLOW_REQUEST_THRESHOLD {
            tracing::trace!(
                request_bytes,
                response_bytes,
                elapsed = ?total,
                reserve = ?reserve,
                exchange = ?exchange,
                recovery = ?recovery,
                nested_handler,
                failed,
                "Slow extension worker-pool request completed."
            );
        }
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.shutdown();
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

fn pool_state_name(state: u8) -> &'static str {
    match state {
        STATE_RUNNING => "running",
        STATE_FINALIZING => "finalizing",
        STATE_STOPPING => "stopping",
        STATE_STOPPED => "stopped",
        _ => "unknown",
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    #[test]
    fn reports_a_missing_worker_program() {
        let command = WorkerCommand::new(OsString::from("mago-extension-worker-that-does-not-exist"));
        let error = WorkerPool::spawn(command, NonZeroUsize::MIN, WorkerPoolOptions::default())
            .expect_err("missing program should fail");

        assert!(matches!(error, WorkerError::Spawn { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn debug_output_redacts_arguments_and_bootstrap_payloads() {
        let pool = WorkerPool::spawn(
            WorkerCommand::new("cat").with_argument("secret-argument"),
            NonZeroUsize::MIN,
            WorkerPoolOptions::default(),
        )
        .expect("worker pool should start");
        lock(&pool.bootstraps)
            .push(Bootstrap { request: b"secret-request".to_vec(), response: b"secret-response".to_vec() });

        let debug = format!("{pool:?}");
        assert!(!debug.contains("secret-argument"));
        assert!(!debug.contains("secret-request"));
        assert!(!debug.contains("secret-response"));
        assert!(debug.contains("bootstrap_count: 1"));

        pool.shutdown();
    }

    #[cfg(unix)]
    #[test]
    fn starts_only_two_workers_eagerly() {
        let pool = WorkerPool::spawn_adaptive(
            WorkerCommand::new("cat"),
            NonZeroUsize::new(4).expect("four is non-zero"),
            WorkerPoolOptions::default(),
        )
        .expect("worker pool should start");

        assert_eq!(pool.len(), 2);
        let running = pool.workers.iter().map(|slot| lock(&slot.worker).is_some()).collect::<Vec<_>>();
        assert_eq!(running, [true, true, false, false]);
        pool.shutdown();
    }

    #[cfg(unix)]
    #[test]
    fn starts_every_worker_in_a_fixed_pool() {
        let pool = WorkerPool::spawn(
            WorkerCommand::new("cat"),
            NonZeroUsize::new(3).expect("three is non-zero"),
            WorkerPoolOptions::default(),
        )
        .expect("worker pool should start");

        assert_eq!(pool.len(), 3);
        assert!(!pool.is_empty());
        assert!(pool.workers.iter().all(|slot| lock(&slot.worker).is_some()));
        pool.shutdown();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn ignores_runtime_warmup_when_deciding_to_expand() {
        let pool = WorkerPool::spawn_adaptive(
            WorkerCommand::new("cat"),
            NonZeroUsize::new(3).expect("three is non-zero"),
            WorkerPoolOptions::default(),
        )
        .expect("worker pool should start");

        for _ in 0..WARMUP_REQUESTS {
            pool.record_request(Duration::from_millis(10));
        }
        assert!(!pool.should_grow(2));

        for _ in 0..(2 * GROWTH_SAMPLE_MULTIPLIER) {
            pool.record_request(Duration::from_millis(10));
        }
        assert!(pool.should_grow(2));
        pool.shutdown();
    }

    #[cfg(unix)]
    #[test]
    fn initializes_a_lazily_started_worker() {
        const WORKER: &str = concat!(
            "dd bs=1 count=36 of=/dev/null 2>/dev/null; ",
            "printf '\\115\\101\\107\\117",              // MAGO
            "\\000\\001\\000\\000",                      // protocol 1.0
            "\\002\\000\\000\\000",                      // response, no flags or reserved bits
            "\\000\\000\\000\\000\\000\\000\\000\\001",  // request id 1
            "\\000\\000\\000\\000\\000\\000\\000\\000",  // no parent
            "\\000\\000\\000\\005",                      // five payload bytes
            "\\162\\145\\141\\144\\171'; ",              // ready
            "dd bs=1 count=32 of=/dev/null 2>/dev/null", // shutdown frame
        );

        let pool = WorkerPool::spawn_adaptive(
            WorkerCommand::new("sh").with_arguments(["-c", WORKER]),
            NonZeroUsize::new(3).expect("three is non-zero"),
            WorkerPoolOptions::default(),
        )
        .expect("worker pool should start");

        assert_eq!(
            pool.broadcast(b"init").expect("initialization should succeed"),
            [b"ready".to_vec(), b"ready".to_vec()]
        );
        let worker = pool.spawn_initialized(2).expect("lazy worker should replay initialization");
        assert!(worker.is_running());

        worker.shutdown();
        pool.shutdown();
    }

    #[cfg(unix)]
    #[test]
    fn starts_requests_and_gracefully_stops_a_real_process() {
        const WORKER: &str = concat!(
            "dd bs=1 count=36 of=/dev/null 2>/dev/null; ",
            "printf '\\115\\101\\107\\117",              // MAGO
            "\\000\\001\\000\\000",                      // protocol 1.0
            "\\002\\000\\000\\000",                      // response, no flags or reserved bits
            "\\000\\000\\000\\000\\000\\000\\000\\001",  // request id 1
            "\\000\\000\\000\\000\\000\\000\\000\\000",  // no parent
            "\\000\\000\\000\\004",                      // four payload bytes
            "\\160\\157\\156\\147'; ",                   // pong
            "dd bs=1 count=32 of=/dev/null 2>/dev/null", // shutdown frame
        );

        let command = WorkerCommand::new("sh").with_arguments(["-c", WORKER]);
        let pool = WorkerPool::spawn(command, NonZeroUsize::MIN, WorkerPoolOptions::default())
            .expect("worker process should start");

        assert_eq!(pool.request(b"ping".to_vec()).expect("request should succeed"), b"pong");
        pool.shutdown();
    }
}
