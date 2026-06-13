use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_flags::U8Flags;

/// `int`, `literal-int`, `int(42)`, `positive-int`, `int<0, 255>`.
///
/// `Unspecified` is the plain `int`. `UnspecifiedLiteral` is "some literal
/// int, value unknown". Negated single values (`int & !int(0)`) are expressed
/// through the intersection machinery rather than a dedicated variant.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IntAtom<'arena> {
    Unspecified,
    UnspecifiedLiteral,
    Literal(i64),
    Range(&'arena IntRange),
}

/// A bounded integer range.
///
/// Either bound may be open (±∞), recorded as [`BoundFlag`]s. When a bound is
/// open, its accompanying value field is canonically zeroed by the constructor
/// so structural equality stays sound.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IntRange {
    lower_value: i64,
    upper_value: i64,
    bounds: U8Flags<BoundFlag>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum BoundFlag {
    Lower = 1 << 0,
    Upper = 1 << 1,
}

impl From<BoundFlag> for u8 {
    fn from(flag: BoundFlag) -> Self {
        flag as u8
    }
}

impl IntRange {
    /// Construct a range. `None` for either bound means open (±∞). The unused
    /// value field is canonicalized to `0` so two ranges with the same
    /// effective bounds always compare equal.
    #[inline]
    #[must_use]
    pub const fn new(lower: Option<i64>, upper: Option<i64>) -> Self {
        let mut bits = 0u8;
        let lower_value = match lower {
            Some(value) => {
                bits |= BoundFlag::Lower as u8;
                value
            }
            None => 0,
        };
        let upper_value = match upper {
            Some(value) => {
                bits |= BoundFlag::Upper as u8;
                value
            }
            None => 0,
        };

        Self { lower_value, upper_value, bounds: U8Flags::from_bits(bits) }
    }

    /// `Some(v)` if the range has a lower bound, `None` if open.
    #[inline]
    #[must_use]
    pub const fn lower(self) -> Option<i64> {
        if self.bounds.contains_bits(BoundFlag::Lower as u8) { Some(self.lower_value) } else { None }
    }

    /// `Some(v)` if the range has an upper bound, `None` if open.
    #[inline]
    #[must_use]
    pub const fn upper(self) -> Option<i64> {
        if self.bounds.contains_bits(BoundFlag::Upper as u8) { Some(self.upper_value) } else { None }
    }

    #[inline]
    #[must_use]
    pub const fn bounds(self) -> U8Flags<BoundFlag> {
        self.bounds
    }
}

impl Display for IntAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            IntAtom::Unspecified => f.write_str("int"),
            IntAtom::UnspecifiedLiteral => f.write_str("literal-int"),
            IntAtom::Literal(value) => write!(f, "int({value})"),
            IntAtom::Range(range) => Display::fmt(range, f),
        }
    }
}

impl Display for IntRange {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match (self.lower(), self.upper()) {
            (Some(1), None) => f.write_str("positive-int"),
            (Some(0), None) => f.write_str("non-negative-int"),
            (Some(lower), None) => write!(f, "int<{lower}, max>"),
            (None, Some(-1)) => f.write_str("negative-int"),
            (None, Some(0)) => f.write_str("non-positive-int"),
            (None, Some(upper)) => write!(f, "int<min, {upper}>"),
            (Some(lower), Some(upper)) => write!(f, "int<{lower}, {upper}>"),
            (None, None) => f.write_str("int"),
        }
    }
}

impl CopyInto for IntAtom<'_> {
    type Output<'arena> = IntAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            IntAtom::Unspecified => IntAtom::Unspecified,
            IntAtom::UnspecifiedLiteral => IntAtom::UnspecifiedLiteral,
            IntAtom::Literal(value) => IntAtom::Literal(value),
            IntAtom::Range(range) => IntAtom::Range(copy_ref_into(range, arena)),
        }
    }
}

impl CopyInto for IntRange {
    type Output<'arena> = IntRange;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}
