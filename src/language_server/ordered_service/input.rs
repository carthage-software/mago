//! Bound encoded request bytes and release each reservation with its handler.

use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use tower_lsp_server::jsonrpc::Request;

// These limits count encoded input bytes, not analyzer caches or response data.
pub(super) const INPUT_BYTES_LIMIT: usize = 16 * 1024 * 1024;
pub(super) const CONTROL_BYTES_LIMIT: usize = 4 * 1024;
const RETAINED_INPUT_BYTES_LIMIT: usize = 64 * 1024 * 1024;

pub(super) struct InputBudget {
    retained: Arc<AtomicUsize>,
    limit: usize,
}

impl Default for InputBudget {
    fn default() -> Self {
        Self::new(RETAINED_INPUT_BYTES_LIMIT)
    }
}

impl InputBudget {
    pub(super) fn new(limit: usize) -> Self {
        Self { retained: Arc::default(), limit }
    }

    pub(super) fn has_capacity(&self, bytes: usize) -> bool {
        self.retained.load(Ordering::Relaxed).checked_add(bytes).is_some_and(|used| used <= self.limit)
    }

    /// Reserve a request's bytes within the limit; a refused request can be retried.
    pub(super) fn reserve(&self, bytes: usize) -> io::Result<InputPermit> {
        self.retained
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                used.checked_add(bytes).filter(|sum| *sum <= self.limit)
            })
            .map_err(|_| io::Error::other("language server input byte budget is full"))?;
        Ok(InputPermit { retained: Arc::clone(&self.retained), bytes })
    }

    /// Count bytes the session must accept: a notification cannot be refused without losing
    /// state, and control messages keep it responsive. They still hold requests back.
    pub(super) fn retain(&self, bytes: usize) -> InputPermit {
        let mut added = bytes;
        let _ = self.retained.try_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
            added = bytes.min(usize::MAX - used);
            Some(used + added)
        });
        InputPermit { retained: Arc::clone(&self.retained), bytes: added }
    }

    #[cfg(test)]
    pub(super) fn retained_bytes(&self) -> usize {
        self.retained.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(super) fn counter(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.retained)
    }
}

pub(super) struct InputPermit {
    retained: Arc<AtomicUsize>,
    bytes: usize,
}

impl Drop for InputPermit {
    fn drop(&mut self) {
        self.retained.fetch_sub(self.bytes, Ordering::Relaxed);
    }
}

pub(super) fn encoded_size(request: &Request, limit: usize) -> io::Result<usize> {
    struct Counter {
        bytes: usize,
        limit: usize,
    }

    impl io::Write for Counter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.bytes = self
                .bytes
                .checked_add(buffer.len())
                .filter(|bytes| *bytes <= self.limit)
                .ok_or_else(|| io::Error::other("language server input is too large"))?;
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut counter = Counter { bytes: 0, limit };
    serde_json::to_writer(&mut counter, request).map_err(io::Error::other)?;
    Ok(counter.bytes)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn encoded_size_counts_escaping_and_unicode_at_the_limit() {
        let request =
            Request::build("textDocument/didChange").params(json!({"contentChanges": [{"text": "\"\n😀\\"}]})).finish();
        let expected = serde_json::to_vec(&request).unwrap().len();
        assert_eq!(encoded_size(&request, expected).unwrap(), expected);
        assert!(encoded_size(&request, expected - 1).is_err());
    }

    #[test]
    fn permits_release_only_their_own_bytes() {
        let budget = InputBudget::new(10);
        let first = budget.reserve(6).unwrap();
        let second = budget.reserve(4).unwrap();
        assert!(budget.reserve(1).is_err());
        assert_eq!(budget.retained_bytes(), 10);
        drop(first);
        assert_eq!(budget.retained_bytes(), 4);
        let replacement = budget.reserve(6).unwrap();
        drop(second);
        assert_eq!(budget.retained_bytes(), 6);
        drop(replacement);
        assert_eq!(budget.retained_bytes(), 0);
    }

    #[test]
    fn retained_bytes_pass_the_limit_and_hold_requests_back() {
        let budget = InputBudget::new(10);
        let request = budget.reserve(10).unwrap();
        let notification = budget.retain(64);
        assert_eq!(budget.retained_bytes(), 74);
        assert!(!budget.has_capacity(0));
        drop(request);
        assert!(budget.reserve(1).is_err());
        drop(notification);
        assert_eq!(budget.retained_bytes(), 0);
        assert!(budget.reserve(10).is_ok());
    }

    #[test]
    fn permits_outlive_the_budget_without_leaking_bytes() {
        let budget = InputBudget::new(10);
        let counter = budget.counter();
        let permit = budget.reserve(10).unwrap();
        drop(budget);
        assert_eq!(counter.load(Ordering::Relaxed), 10);
        drop(permit);
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn reservations_cannot_overflow_the_counter() {
        let budget = InputBudget::new(usize::MAX);
        let permit = budget.reserve(usize::MAX).unwrap();
        assert!(budget.reserve(1).is_err());
        let retained = budget.retain(1);
        assert_eq!(budget.retained_bytes(), usize::MAX);
        drop(retained);
        assert_eq!(budget.retained_bytes(), usize::MAX);
        drop(permit);
        assert_eq!(budget.retained_bytes(), 0);
    }
}
