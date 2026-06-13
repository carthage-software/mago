#[cfg(feature = "serde")]
use serde::Serialize;

use crate::ty::atom::kind::AtomKind;

/// A bitset over [`AtomKind`]s.
///
/// Every [`Type`](crate::Type) carries the set of kinds present in its union,
/// computed once at construction, so "does this type contain an atom of kind
/// K?" is a single mask test with no slice traversal.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct AtomKindSet(u64);

impl AtomKindSet {
    pub const EMPTY: Self = Self(0);

    #[inline]
    #[must_use]
    pub const fn of(kind: AtomKind) -> Self {
        Self(1 << kind.discriminant())
    }

    #[inline]
    #[must_use]
    pub const fn with(self, kind: AtomKind) -> Self {
        Self(self.0 | (1 << kind.discriminant()))
    }

    #[inline]
    #[must_use]
    pub const fn contains(self, kind: AtomKind) -> bool {
        self.0 & (1 << kind.discriminant()) != 0
    }

    #[inline]
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    #[inline]
    #[must_use]
    pub const fn is_subset_of(self, other: Self) -> bool {
        self.0 & !other.0 == 0
    }

    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    #[must_use]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }
}
