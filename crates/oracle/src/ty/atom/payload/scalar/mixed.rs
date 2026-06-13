use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_flags::U8Flags;

/// Refinements on `mixed`: nullability, emptiness, truthiness, and the
/// isset-from-loop marker.
///
/// The flags are wrapped rather than exposed because truthiness is a
/// tri-state: [`MixedFlag::Truthy`] and [`MixedFlag::Falsy`] are mutually
/// exclusive, and [`with_truthiness`](Self::with_truthiness) is the only way
/// to set either.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct MixedAtom(U8Flags<MixedFlag>);

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum MixedFlag {
    NonNull = 1 << 0,
    Empty = 1 << 1,
    Truthy = 1 << 2,
    Falsy = 1 << 3,
    IssetFromLoop = 1 << 4,
}

impl From<MixedFlag> for u8 {
    fn from(flag: MixedFlag) -> Self {
        flag as u8
    }
}

/// Three-state truthiness for [`MixedAtom`].
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Truthiness {
    #[default]
    Undetermined,
    Truthy,
    Falsy,
}

const fn with_bit(flags: U8Flags<MixedFlag>, flag: MixedFlag, on: bool) -> U8Flags<MixedFlag> {
    let bit = flag as u8;

    U8Flags::from_bits(if on { flags.bits() | bit } else { flags.bits() & !bit })
}

impl MixedAtom {
    pub const EMPTY: Self = Self(U8Flags::empty());

    #[inline]
    #[must_use]
    pub const fn truthiness(self) -> Truthiness {
        if self.0.contains_bits(MixedFlag::Truthy as u8) {
            Truthiness::Truthy
        } else if self.0.contains_bits(MixedFlag::Falsy as u8) {
            Truthiness::Falsy
        } else {
            Truthiness::Undetermined
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_non_null(self) -> bool {
        self.0.contains_bits(MixedFlag::NonNull as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0.contains_bits(MixedFlag::Empty as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_isset_from_loop(self) -> bool {
        self.0.contains_bits(MixedFlag::IssetFromLoop as u8)
    }

    #[inline]
    #[must_use]
    pub const fn with_is_non_null(self, on: bool) -> Self {
        Self(with_bit(self.0, MixedFlag::NonNull, on))
    }

    #[inline]
    #[must_use]
    pub const fn with_is_empty(self, on: bool) -> Self {
        Self(with_bit(self.0, MixedFlag::Empty, on))
    }

    #[inline]
    #[must_use]
    pub const fn with_truthiness(self, truthiness: Truthiness) -> Self {
        let cleared = with_bit(with_bit(self.0, MixedFlag::Truthy, false), MixedFlag::Falsy, false);

        Self(match truthiness {
            Truthiness::Undetermined => cleared,
            Truthiness::Truthy => with_bit(cleared, MixedFlag::Truthy, true),
            Truthiness::Falsy => with_bit(cleared, MixedFlag::Falsy, true),
        })
    }

    #[inline]
    #[must_use]
    pub const fn with_is_isset_from_loop(self, on: bool) -> Self {
        Self(with_bit(self.0, MixedFlag::IssetFromLoop, on))
    }
}

impl Display for MixedAtom {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let label = if self.is_empty() {
            match self.truthiness() {
                Truthiness::Truthy => "empty-truthy-mixed",
                Truthiness::Falsy => "empty-falsy-mixed",
                Truthiness::Undetermined if self.is_non_null() => "empty-nonnull",
                Truthiness::Undetermined => "empty-mixed",
            }
        } else {
            match self.truthiness() {
                Truthiness::Truthy => "truthy-mixed",
                Truthiness::Falsy => "falsy-mixed",
                Truthiness::Undetermined if self.is_non_null() => "nonnull",
                Truthiness::Undetermined => "mixed",
            }
        };

        f.write_str(label)
    }
}

impl CopyInto for MixedAtom {
    type Output<'arena> = MixedAtom;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}
