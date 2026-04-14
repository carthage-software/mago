//! Pipeline telemetry helpers gated on the `TRACE` log level.

use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use mago_database::file::File;

/// Measures `body`, storing the elapsed time in `out` when `trace_enabled`
/// is true. When it's false, the work still runs but no clock reads or
/// stores happen, giving the branch predictor a hot path with no
/// measurement cost.
macro_rules! measure {
    ($trace_enabled:expr, $out:expr, $body:expr) => {{
        if $trace_enabled {
            let __start = ::std::time::Instant::now();
            let __result = $body;
            $out = __start.elapsed();
            __result
        } else {
            $body
        }
    }};
}

pub(crate) use measure;

/// Per-phase aggregate counters for the analyze-parallel hot loop.
///
/// All fields are nanosecond sums across every worker thread; divide by
/// number of files for the per-file average, or by number of threads for an
/// approximation of wall-time contribution.
#[derive(Default)]
pub(crate) struct AnalysisPhaseTelemetry {
    pub(crate) parse_ns: AtomicU64,
    pub(crate) resolve_ns: AtomicU64,
    pub(crate) analyzer_new_ns: AtomicU64,
    pub(crate) semantics_ns: AtomicU64,
    pub(crate) analyze_ns: AtomicU64,
    pub(crate) per_file_total_ns: AtomicU64,
    pub(crate) files: AtomicU64,
}

impl AnalysisPhaseTelemetry {
    pub(crate) fn dump(&self) {
        let files = self.files.load(Relaxed);
        if files == 0 {
            return;
        }

        let per_file_us = |counter: &AtomicU64| -> f64 { (counter.load(Relaxed) as f64 / files as f64) / 1_000.0 };
        let total_ms = |counter: &AtomicU64| -> f64 { counter.load(Relaxed) as f64 / 1_000_000.0 };

        tracing::trace!("Analyzed {} files in the parallel phase.", files);
        tracing::trace!(
            "Parsing accounted for {:.3} ms of worker CPU time (average {:.2} µs per file).",
            total_ms(&self.parse_ns),
            per_file_us(&self.parse_ns),
        );
        tracing::trace!(
            "Name resolution accounted for {:.3} ms of worker CPU time (average {:.2} µs per file).",
            total_ms(&self.resolve_ns),
            per_file_us(&self.resolve_ns),
        );
        tracing::trace!(
            "Analyzer construction accounted for {:.3} ms of worker CPU time (average {:.2} µs per file).",
            total_ms(&self.analyzer_new_ns),
            per_file_us(&self.analyzer_new_ns),
        );
        tracing::trace!(
            "Semantics checking accounted for {:.3} ms of worker CPU time (average {:.2} µs per file).",
            total_ms(&self.semantics_ns),
            per_file_us(&self.semantics_ns),
        );
        tracing::trace!(
            "Full analysis (Analyzer::analyze) accounted for {:.3} ms of worker CPU time (average {:.2} µs per file).",
            total_ms(&self.analyze_ns),
            per_file_us(&self.analyze_ns),
        );
        tracing::trace!(
            "Total worker CPU time across all sub-phases was {:.3} ms (average {:.2} µs per file).",
            total_ms(&self.per_file_total_ns),
            per_file_us(&self.per_file_total_ns),
        );
    }
}

/// Collects per-file durations from workers in a parallel phase so that at
/// the end of the phase the N slowest files can be reported.
pub(crate) struct SlowestFiles {
    entries: Mutex<Vec<(Duration, Arc<File>)>>,
}

impl SlowestFiles {
    pub(crate) fn new() -> Self {
        Self { entries: Mutex::new(Vec::new()) }
    }

    #[inline]
    pub(crate) fn record(&self, duration: Duration, file: Arc<File>) {
        if let Ok(mut guard) = self.entries.lock() {
            guard.push((duration, file));
        }
    }

    /// Sorts the collected entries and emits the slowest `limit` via
    /// `tracing::trace!`. One event per file, as a natural sentence.
    pub(crate) fn emit_slowest(&self, limit: usize, phase: &str) {
        let Ok(mut guard) = self.entries.lock() else {
            return;
        };

        if guard.is_empty() {
            return;
        }

        guard.sort_by_key(|b| std::cmp::Reverse(b.0));

        let shown = guard.len().min(limit);
        tracing::trace!("Slowest {} of {} files during {}:", shown, guard.len(), phase);
        for (rank, (duration, file)) in guard.iter().take(shown).enumerate() {
            tracing::trace!("  {:>2}. {} took {:?}.", rank + 1, file.name, duration);
        }
    }
}

const HANG_INITIAL_THRESHOLD: Duration = Duration::from_millis(5000);
const HANG_POLL_INTERVAL: Duration = Duration::from_secs(2);

struct InflightEntry {
    file: Arc<File>,
    started_at: Instant,
    next_report_at: Instant,
    reported: bool,
}

/// Watches in-flight analyzer workers and emits a trace when a file has been
/// analyzing for longer than a threshold.
///
/// Intended for diagnosing pathological inputs that put the analyzer in a long
/// or infinite loop; without this, the only trace we emit for a file is after
/// it finishes, which never happens for a hang.
pub(crate) struct HangWatcher {
    slots: Arc<Vec<Mutex<Option<InflightEntry>>>>,
    shutdown: Arc<AtomicBool>,
    cv: Arc<(Mutex<()>, Condvar)>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl HangWatcher {
    pub(crate) fn spawn(num_slots: usize) -> Arc<Self> {
        let slots: Arc<Vec<Mutex<Option<InflightEntry>>>> =
            Arc::new((0..num_slots.max(1)).map(|_| Mutex::new(None)).collect());
        let shutdown = Arc::new(AtomicBool::new(false));
        let cv = Arc::new((Mutex::new(()), Condvar::new()));

        let watcher_slots = Arc::clone(&slots);
        let watcher_shutdown = Arc::clone(&shutdown);
        let watcher_cv = Arc::clone(&cv);
        let handle = std::thread::Builder::new()
            .name("mago-hang-watcher".into())
            .spawn(move || Self::run(watcher_slots, watcher_shutdown, watcher_cv))
            .expect("failed to spawn hang watcher thread");

        Arc::new(Self { slots, shutdown, cv, handle: Mutex::new(Some(handle)) })
    }

    fn run(slots: Arc<Vec<Mutex<Option<InflightEntry>>>>, shutdown: Arc<AtomicBool>, cv: Arc<(Mutex<()>, Condvar)>) {
        let (lock, cvar) = &*cv;
        loop {
            if shutdown.load(Relaxed) {
                return;
            }

            let guard = lock.lock().unwrap();
            let (_guard, _) = cvar.wait_timeout(guard, HANG_POLL_INTERVAL).unwrap();

            if shutdown.load(Relaxed) {
                return;
            }

            let now = Instant::now();
            for slot in slots.iter() {
                let Ok(mut entry_guard) = slot.lock() else {
                    continue;
                };

                let Some(entry) = entry_guard.as_mut() else {
                    continue;
                };

                if now < entry.next_report_at {
                    continue;
                }

                let elapsed = now - entry.started_at;
                let secs = elapsed.as_secs();

                if entry.reported {
                    tracing::trace!("{} is still being analyzed after {secs}s.", entry.file.name);
                } else {
                    tracing::trace!("{} has been analyzing for {secs}s and has not finished.", entry.file.name);
                    tracing::trace!(
                        "No file should take this long to analyze. This is almost certainly a bug in mago."
                    );
                    tracing::trace!(
                        "Please report it at https://github.com/carthage-software/mago/issues/new and attach {} if you can share it.",
                        entry.file.name,
                    );
                    tracing::trace!(
                        "If the file is private, anonymize it (rename identifiers, remove sensitive literals) before attaching."
                    );
                    entry.reported = true;
                }

                entry.next_report_at = now + elapsed.max(HANG_INITIAL_THRESHOLD);
            }
        }
    }

    /// Records that the current rayon worker has started analyzing `file` and
    /// returns a guard that clears the entry when dropped.
    pub(crate) fn track(self: &Arc<Self>, file: Arc<File>) -> HangGuard {
        let idx = rayon::current_thread_index().unwrap_or(0);
        if idx < self.slots.len()
            && let Ok(mut slot) = self.slots[idx].lock()
        {
            let now = Instant::now();
            *slot = Some(InflightEntry {
                file,
                started_at: now,
                next_report_at: now + HANG_INITIAL_THRESHOLD,
                reported: false,
            });
        }

        HangGuard { slots: Arc::clone(&self.slots), idx }
    }
}

impl Drop for HangWatcher {
    fn drop(&mut self) {
        self.shutdown.store(true, Relaxed);
        let (_, cvar) = &*self.cv;
        cvar.notify_all();
        if let Ok(mut handle_guard) = self.handle.lock()
            && let Some(handle) = handle_guard.take()
        {
            let _ = handle.join();
        }
    }
}

pub(crate) struct HangGuard {
    slots: Arc<Vec<Mutex<Option<InflightEntry>>>>,
    idx: usize,
}

impl Drop for HangGuard {
    fn drop(&mut self) {
        if self.idx < self.slots.len()
            && let Ok(mut slot) = self.slots[self.idx].lock()
        {
            *slot = None;
        }
    }
}
