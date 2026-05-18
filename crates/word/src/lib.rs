//! A byte-string interning library for the Mago ecosystem.
//!
//! Provides [`Word`], a `Copy` handle that backs every interned byte sequence. Two
//! `Word`s with identical content always compare equal and hash equal; their bit
//! pattern is canonical, which keeps equality and hashing in O(1).
//!
//! On top of [`Word`] this crate provides a collection of zero-allocation utilities
//! mirroring the `mago_atom` API surface — ASCII case folding, fully-qualified-name
//! lowercasing, SIMD-accelerated case-insensitive prefix matching, integer/float
//! formatting, and a [`concat_word!`] macro for stack-allocated concatenation.
//!
//! # Features
//!
//! - `sso` *(default)* — enables small-string optimization. With this on, sequences
//!   up to [`INLINE_CAPACITY`] are stored inline in the 16-byte handle and never
//!   touch the global arena. Disabling this feature falls back to pure interning,
//!   where every `Word` is a pointer.
//!
//! # Example
//!
//! ```
//! use mago_word::Word;
//! use mago_word::ascii_lowercase_word;
//!
//! let s1 = Word::new(b"Hello");
//! let s2 = ascii_lowercase_word(b"Hello");
//! assert_eq!(s2.as_bytes(), b"hello");
//! ```

#![deny(unsafe_op_in_unsafe_fn)]

mod bumpalloc;
mod interner;
mod util;
mod word;

use std::collections::HashMap;
use std::collections::HashSet;

use foldhash::fast::FixedState;

pub use crate::util::*;
#[cfg(feature = "sso")]
pub use crate::word::INLINE_CAPACITY;
pub use crate::word::Word;

/// A `HashMap` keyed by [`Word`], using a fast non-cryptographic hasher.
pub type WordMap<V> = HashMap<Word, V, FixedState>;

/// A `HashSet` of [`Word`]s, using a fast non-cryptographic hasher.
pub type WordSet = HashSet<Word, FixedState>;

/// Interns `bytes` and returns its canonical [`Word`]. Convenience wrapper over
/// [`Word::new`].
#[inline]
#[must_use]
pub fn word(bytes: impl AsRef<[u8]>) -> Word {
    Word::new(bytes.as_ref())
}
