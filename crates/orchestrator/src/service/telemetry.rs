//! Pipeline telemetry helpers gated on the `TRACE` log level.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

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
