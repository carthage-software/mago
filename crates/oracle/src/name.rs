use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

/// A symbol name: class-like, function, method, property, constant, template
/// parameter, or variable.
///
/// Names are exact byte sequences; equality, ordering, and hashing are all
/// byte-wise. PHP's case-insensitive lookups (class and function names) are a
/// resolution concern that belongs to [`World`](crate::world) implementations,
/// never to name identity.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name<'arena>(&'arena [u8]);

impl<'arena> Name<'arena> {
    #[inline]
    #[must_use]
    pub const fn new(bytes: &'arena [u8]) -> Self {
        Self(bytes)
    }

    #[inline]
    #[must_use]
    pub const fn as_bytes(self) -> &'arena [u8] {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn as_str_lossy(self) -> Cow<'arena, str> {
        String::from_utf8_lossy(self.0)
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    #[must_use]
    pub const fn len(self) -> usize {
        self.0.len()
    }
}

impl Display for Name<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.as_str_lossy())
    }
}

impl CopyInto for Name<'_> {
    type Output<'arena> = Name<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Name(arena.alloc_slice_copy(self.0))
    }
}
