use serde::Deserialize;
use serde::Serialize;

/// Flags representing various properties of a type union.
///
/// This replaces 9 individual boolean fields with a compact 16-bit representation,
/// reducing memory usage from 9 bytes to 2 bytes per TUnion instance.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnionFlags(u16);

impl UnionFlags {
    /// Indicates the union had a template type at some point.
    pub const HAD_TEMPLATE: UnionFlags = UnionFlags(1 << 0);
    /// Indicates the value is passed by reference.
    pub const BY_REFERENCE: UnionFlags = UnionFlags(1 << 1);
    /// Indicates no references exist to this type.
    pub const REFERENCE_FREE: UnionFlags = UnionFlags(1 << 2);
    /// Indicates the type may be undefined due to a try block.
    pub const POSSIBLY_UNDEFINED_FROM_TRY: UnionFlags = UnionFlags(1 << 3);
    /// Indicates the type may be undefined.
    pub const POSSIBLY_UNDEFINED: UnionFlags = UnionFlags(1 << 4);
    /// Indicates nullable issues should be ignored for this type.
    pub const IGNORE_NULLABLE_ISSUES: UnionFlags = UnionFlags(1 << 5);
    /// Indicates falsable issues should be ignored for this type.
    pub const IGNORE_FALSABLE_ISSUES: UnionFlags = UnionFlags(1 << 6);
    /// Indicates the type came from a template default value.
    pub const FROM_TEMPLATE_DEFAULT: UnionFlags = UnionFlags(1 << 7);
    /// Indicates the type has been populated with codebase information.
    pub const POPULATED: UnionFlags = UnionFlags(1 << 8);
    /// Indicates the null in this union came from nullsafe short-circuit.
    pub const NULLSAFE_NULL: UnionFlags = UnionFlags(1 << 9);
}

impl UnionFlags {
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        UnionFlags(0)
    }

    #[inline]
    pub const fn insert(&mut self, flag: UnionFlags) {
        self.0 |= flag.0;
    }

    #[inline]
    pub const fn set(&mut self, flag: UnionFlags, value: bool) {
        if value {
            self.insert(flag);
        } else {
            self.0 &= !flag.0;
        }
    }

    #[inline]
    pub const fn contains(self, flag: UnionFlags) -> bool {
        (self.0 & flag.0) == flag.0
    }

    #[inline]
    pub const fn intersects(self, other: UnionFlags) -> bool {
        (self.0 & other.0) != 0
    }

    #[inline]
    #[must_use]
    pub const fn union(&self, other: UnionFlags) -> UnionFlags {
        UnionFlags(self.0 | other.0)
    }

    #[inline]
    #[must_use]
    pub const fn intersection(&self, other: UnionFlags) -> UnionFlags {
        UnionFlags(self.0 & other.0)
    }
}

impl std::ops::BitOr for UnionFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        UnionFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for UnionFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for UnionFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        UnionFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for UnionFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::Not for UnionFlags {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        UnionFlags(!self.0)
    }
}
