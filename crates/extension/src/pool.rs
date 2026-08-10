use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::command::WorkerCommand;
use crate::error::WorkerError;
use crate::worker::Worker;
use crate::worker::WorkerRequestHandler;

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
    worker: Mutex<Arc<Worker>>,
    restart: Mutex<()>,
}

/// A fixed-size pool of persistent, multiplexed extension worker processes.
///
/// Scheduling prefers the live worker with the fewest requests in flight.
/// Equal loads are distributed round-robin. CPU-bound parallelism comes from
/// the process count, while each process may cooperatively interleave multiple
/// requests when its runtime supports that.
pub struct WorkerPool {
    command: WorkerCommand,
    options: WorkerPoolOptions,
    workers: Box<[WorkerSlot]>,
    cursor: AtomicUsize,
    shutting_down: AtomicBool,
}

impl std::fmt::Debug for WorkerPool {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkerPool")
            .field("command", &self.command)
            .field("options", &self.options)
            .field("worker_count", &self.workers.len())
            .field("cursor", &self.cursor.load(Ordering::Relaxed))
            .field("shutting_down", &self.shutting_down.load(Ordering::Relaxed))
            .finish()
    }
}

impl WorkerPool {
    /// Starts `size` identical worker processes.
    ///
    /// # Errors
    ///
    /// Returns an error if any worker process or one of its stream reader
    /// threads cannot be started.
    pub fn spawn(command: WorkerCommand, size: NonZeroUsize, options: WorkerPoolOptions) -> Result<Self, WorkerError> {
        let mut workers = Vec::with_capacity(size.get());
        for id in 0..size.get() {
            let worker = Worker::spawn(id, &command, &options)?;
            workers.push(WorkerSlot { worker: Mutex::new(worker), restart: Mutex::new(()) });
        }

        Ok(Self {
            command,
            options,
            workers: workers.into_boxed_slice(),
            cursor: AtomicUsize::new(0),
            shutting_down: AtomicBool::new(false),
        })
    }

    /// Number of processes managed by this pool.
    #[must_use]
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// A worker pool always contains at least one process.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// Sends a request through the least-loaded worker and blocks for its response.
    ///
    /// # Errors
    ///
    /// Returns an error when no worker is available, communication fails, the
    /// deadline expires, or the worker returns an error response.
    pub fn request(&self, payload: Vec<u8>) -> Result<Vec<u8>, WorkerError> {
        let (index, reservation) = self.reserve_worker()?;
        let result = reservation.worker().request(payload);
        self.recover_if_needed(index, &reservation, &result)?;
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
        let (index, reservation) = self.reserve_worker()?;
        let result = reservation.worker().request_with_handler(payload, handler);
        self.recover_if_needed(index, &reservation, &result)?;
        result
    }

    /// Sends the same request to every worker concurrently and returns results
    /// in worker-index order.
    ///
    /// This is intended for initialization and registration validation, where
    /// every process in a pool must begin with identical extension state.
    ///
    /// # Errors
    ///
    /// Returns an error if any worker cannot be started, communicated with, or
    /// restarted after failure. All coordinator threads are joined before the
    /// error is returned.
    pub fn broadcast(&self, payload: &[u8]) -> Result<Vec<Vec<u8>>, WorkerError> {
        if self.shutting_down.load(Ordering::Acquire) {
            return Err(WorkerError::Unavailable);
        }

        let workers = (0..self.workers.len())
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

        let mut responses = vec![None; self.workers.len()];
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
            return Err(error);
        }

        Ok(responses.into_iter().flatten().collect())
    }

    /// Gracefully stops every worker, then kills workers that exceed the
    /// configured shutdown grace period.
    pub fn shutdown(&self) {
        if self.shutting_down.swap(true, Ordering::AcqRel) {
            return;
        }

        let workers: Vec<_> = self.workers.iter().map(|slot| Arc::clone(&lock(&slot.worker))).collect();
        for worker in &workers {
            worker.begin_shutdown();
        }

        let deadline = std::time::Instant::now() + self.options.shutdown_timeout;
        for worker in workers {
            worker.finish_shutdown(deadline);
        }
    }

    fn reserve_worker(&self) -> Result<(usize, crate::worker::WorkerReservation), WorkerError> {
        if self.shutting_down.load(Ordering::Acquire) {
            return Err(WorkerError::Unavailable);
        }

        let start = self.cursor.fetch_add(1, Ordering::Relaxed) % self.workers.len();
        let mut selected: Option<(usize, Arc<Worker>, usize)> = None;
        let mut last_error = None;

        for offset in 0..self.workers.len() {
            let index = (start + offset) % self.workers.len();
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

        let Some((index, worker, _)) = selected else {
            return Err(last_error.unwrap_or(WorkerError::Unavailable));
        };

        Ok((index, worker.reserve()))
    }

    fn ensure_running(&self, index: usize) -> Result<Arc<Worker>, WorkerError> {
        let worker = Arc::clone(&lock(&self.workers[index].worker));
        if worker.is_running() {
            return Ok(worker);
        }

        self.restart(index, &worker)
    }

    fn recover_if_needed(
        &self,
        index: usize,
        reservation: &crate::worker::WorkerReservation,
        result: &Result<Vec<u8>, WorkerError>,
    ) -> Result<(), WorkerError> {
        if result.is_ok() || reservation.worker().is_running() || self.shutting_down.load(Ordering::Acquire) {
            return Ok(());
        }

        let failed = Arc::clone(&lock(&self.workers[index].worker));
        if reservation.matches(&failed) {
            self.restart(index, &failed)?;
        }

        Ok(())
    }

    fn restart(&self, index: usize, failed: &Arc<Worker>) -> Result<Arc<Worker>, WorkerError> {
        let _restart = lock(&self.workers[index].restart);
        let mut slot = lock(&self.workers[index].worker);
        if !Arc::ptr_eq(&slot, failed) && slot.is_running() {
            return Ok(Arc::clone(&slot));
        }

        failed.shutdown();
        let replacement = Worker::spawn(index, &self.command, &self.options)
            .map_err(|source| WorkerError::Restart { worker: index, source: Box::new(source) })?;
        *slot = Arc::clone(&replacement);

        Ok(replacement)
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
