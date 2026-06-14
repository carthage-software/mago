use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use ordered_float::OrderedFloat;
#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

/// `float`, `literal-float`, `float(1.5)`.
///
/// `is_nan` semantics are an analyzer concern, not the type system's.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FloatAtom {
    Unspecified,
    UnspecifiedLiteral,
    Literal(LiteralFloat),
}

/// Newtype around `OrderedFloat<f64>` so the public API doesn't leak the
/// `ordered_float` crate at every callsite.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LiteralFloat(pub OrderedFloat<f64>);

impl LiteralFloat {
    #[inline]
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(OrderedFloat(value))
    }

    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0.0
    }
}

impl Display for FloatAtom {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            FloatAtom::Unspecified => f.write_str("float"),
            FloatAtom::UnspecifiedLiteral => f.write_str("literal-float"),
            FloatAtom::Literal(literal) => write!(f, "float({})", literal.value()),
        }
    }
}

impl CopyInto for FloatAtom {
    type Output<'arena> = FloatAtom;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}
