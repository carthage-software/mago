#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::assertion::Assertion;
use crate::var::Var;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum FunctionLikeAssertionFlag {
    IfTrue = 1 << 0,
    IfFalse = 1 << 1,
    Inferred = 1 << 2,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum FunctionLikeAssertionTarget<'arena> {
    Parameter(Var<'arena>),
    Property(Var<'arena>, &'arena [u8]),
    Method(Var<'arena>, &'arena [u8]),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct FunctionLikeAssertion<'arena> {
    pub span: Span,
    pub flags: U8Flags<FunctionLikeAssertionFlag>,
    pub target: FunctionLikeAssertionTarget<'arena>,
    pub assertion: Assertion<'arena>,
}

impl FunctionLikeAssertion<'_> {
    #[inline]
    #[must_use]
    pub const fn is_if_true(&self) -> bool {
        self.flags.contains_bits(FunctionLikeAssertionFlag::IfTrue as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_if_false(&self) -> bool {
        self.flags.contains_bits(FunctionLikeAssertionFlag::IfFalse as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_inferred(&self) -> bool {
        self.flags.contains_bits(FunctionLikeAssertionFlag::Inferred as u8)
    }
}

impl From<FunctionLikeAssertionFlag> for u8 {
    #[inline]
    fn from(flag: FunctionLikeAssertionFlag) -> Self {
        flag as u8
    }
}

impl HasSpan for FunctionLikeAssertion<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
