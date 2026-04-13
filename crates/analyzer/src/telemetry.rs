//! Aggregated per-phase timing for the analyzer, gated on the `TRACE` log level.

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;

static SETUP_NS: AtomicU64 = AtomicU64::new(0);
static STATEMENTS_NS: AtomicU64 = AtomicU64::new(0);
static FINISH_NS: AtomicU64 = AtomicU64::new(0);
static FILES: AtomicU64 = AtomicU64::new(0);

#[inline]
pub(crate) fn record_setup(duration: Duration) {
    if tracing::enabled!(tracing::Level::TRACE) {
        SETUP_NS.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }
}

#[inline]
pub(crate) fn record_statements(duration: Duration) {
    if tracing::enabled!(tracing::Level::TRACE) {
        STATEMENTS_NS.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }
}

#[inline]
pub(crate) fn record_finish(duration: Duration) {
    if tracing::enabled!(tracing::Level::TRACE) {
        FINISH_NS.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }
}

#[inline]
pub(crate) fn record_file() {
    if tracing::enabled!(tracing::Level::TRACE) {
        FILES.fetch_add(1, Ordering::Relaxed);
    }
}

/// Emits the aggregated breakdown via `tracing::trace!` and resets the
/// counters to zero. No-op when `TRACE` is not enabled.
pub fn dump_and_reset() {
    if !tracing::enabled!(tracing::Level::TRACE) {
        return;
    }

    let files = FILES.swap(0, Ordering::Relaxed);
    let setup_ns = SETUP_NS.swap(0, Ordering::Relaxed);
    let statements_ns = STATEMENTS_NS.swap(0, Ordering::Relaxed);
    let finish_ns = FINISH_NS.swap(0, Ordering::Relaxed);

    if files == 0 {
        return;
    }

    let per_file_us = |ns: u64| -> f64 { (ns as f64 / files as f64) / 1_000.0 };
    let total_ms = |ns: u64| -> f64 { ns as f64 / 1_000_000.0 };

    tracing::trace!(
        "Analyzer setup accounted for {:.3} ms of CPU time (average {:.2} µs per file).",
        total_ms(setup_ns),
        per_file_us(setup_ns),
    );

    tracing::trace!(
        "Statement analysis accounted for {:.3} ms of CPU time (average {:.2} µs per file).",
        total_ms(statements_ns),
        per_file_us(statements_ns),
    );

    tracing::trace!(
        "Analyzer finalization accounted for {:.3} ms of CPU time (average {:.2} µs per file).",
        total_ms(finish_ns),
        per_file_us(finish_ns),
    );
}
