//! Shared primitives for parser token streams.
//!
//! Every syntax crate in the workspace runs the same lookahead pattern:
//! pull tokens from a lexer into a small ring buffer, serve up `peek(n)`
//! / `consume` / `pop_front` against it, and never shrink back to zero.
//! This module owns the buffer so there is one tuned implementation for
//! all four grammars.

use std::fmt::Debug;
use std::mem::MaybeUninit;

/// Fixed-capacity ring buffer for parser lookahead.
///
/// Slots are [`MaybeUninit`] because the token type is expected to be
/// [`Copy`] and the buffer tracks occupancy through `head` and `len`.
/// Only `head..head + len` indices (modulo `CAP`) are considered
/// initialised at any moment.
///
/// `CAP` **must be a power of two**; the indexing arithmetic uses a
/// bitmask (`& (CAP - 1)`) instead of a modulo for speed. A debug-build
/// assertion rejects non-power-of-two capacities during construction.
pub struct LookaheadBuf<T: Copy, const CAP: usize> {
    slots: [MaybeUninit<T>; CAP],
    head: usize,
    len: usize,
}

impl<T: Copy, const CAP: usize> Debug for LookaheadBuf<T, CAP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LookaheadBuf").field("cap", &CAP).field("head", &self.head).field("len", &self.len).finish()
    }
}

impl<T: Copy, const CAP: usize> Default for LookaheadBuf<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const CAP: usize> LookaheadBuf<T, CAP> {
    /// Create an empty buffer. `CAP` must be a power of two; violating
    /// this in debug builds trips a panic on first use.
    #[inline(always)]
    pub fn new() -> Self {
        debug_assert!(CAP.is_power_of_two(), "LookaheadBuf CAP must be a power of two");
        Self { slots: [const { MaybeUninit::uninit() }; CAP], head: 0, len: 0 }
    }

    /// Number of tokens currently buffered.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the buffer is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether the buffer is full. Callers that rely on the ring never
    /// overflowing (no expansion is possible) should check this before
    /// pushing.
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.len == CAP
    }

    /// Capacity, identical to the `CAP` const generic.
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Push a token at the back.
    ///
    /// Hard-panics on overflow: the ring has no heap fallback and
    /// overwriting the oldest slot would silently corrupt the buffer.
    /// Callers must size `CAP` large enough for the deepest lookahead
    /// the parser ever performs.
    #[inline(always)]
    pub fn push_back(&mut self, value: T) {
        assert!(self.len < CAP, "LookaheadBuf overflow: pushed {CAP} tokens without consuming");
        let idx = (self.head + self.len) & (CAP - 1);
        self.slots[idx] = MaybeUninit::new(value);
        self.len += 1;
    }

    /// Pop the front token, or `None` if the buffer is empty.
    #[inline(always)]
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: `head` is within `len` occupied slots by construction;
        // those slots are always written before being read.
        let value = unsafe { self.slots[self.head].assume_init() };
        self.head = (self.head + 1) & (CAP - 1);
        self.len -= 1;
        Some(value)
    }

    /// Copy the `n`th-ahead token without consuming it (0 = next).
    #[inline(always)]
    pub fn get(&self, n: usize) -> Option<T> {
        if n >= self.len {
            return None;
        }
        let idx = (self.head + n) & (CAP - 1);
        // SAFETY: `n < self.len` and all slots in [head, head+len) are
        // initialised by construction.
        Some(unsafe { self.slots[idx].assume_init() })
    }

    /// Discard every buffered token. Leaves the ring ready for reuse.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::LookaheadBuf;

    #[test]
    fn roundtrip_smoke() {
        let mut buf: LookaheadBuf<u32, 8> = LookaheadBuf::new();
        assert!(buf.is_empty());
        for i in 0..6 {
            buf.push_back(i);
        }
        assert_eq!(buf.len(), 6);
        assert_eq!(buf.get(0), Some(0));
        assert_eq!(buf.get(5), Some(5));
        assert_eq!(buf.get(6), None);
        for i in 0..6 {
            assert_eq!(buf.pop_front(), Some(i));
        }
        assert!(buf.is_empty());
    }

    #[test]
    fn wraps_around() {
        let mut buf: LookaheadBuf<u32, 4> = LookaheadBuf::new();
        for i in 0..4 {
            buf.push_back(i);
        }
        assert!(buf.is_full());
        for _ in 0..3 {
            buf.pop_front();
        }
        assert_eq!(buf.len(), 1);
        buf.push_back(99);
        buf.push_back(100);
        assert_eq!(buf.get(0), Some(3));
        assert_eq!(buf.get(1), Some(99));
        assert_eq!(buf.get(2), Some(100));
    }
}
