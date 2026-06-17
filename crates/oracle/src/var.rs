use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::hash::Hash;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Var<'arena>(&'arena [u8]);

impl<'arena> Var<'arena> {
    #[inline]
    #[must_use]
    pub const fn new(name: &'arena [u8]) -> Self {
        Self(name)
    }

    #[inline]
    #[must_use]
    pub const fn name(self) -> &'arena [u8] {
        self.0
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

impl Display for Var<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.as_str_lossy())
    }
}

impl CopyInto for Var<'_> {
    type Output<'arena> = Var<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Var(arena.alloc_slice_copy(self.0))
    }
}
