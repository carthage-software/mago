#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ConstantFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    Polyfill = 1 << 4,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ConstantSymbol<'arena> {
    pub span: Span,
    pub name: Path<'arena>,
    pub attributes: &'arena [AppliedAttribute<'arena>],
    pub flags: U8Flags<ConstantFlag>,
    pub constraint: SymbolConstraint<'arena>,
    pub ty: TypeSlot<'arena>,
    pub origin: Origin,
}

impl<'arena> ConstantSymbol<'arena> {
    #[inline]
    #[must_use]
    pub const fn new(
        span: Span,
        name: Path<'arena>,
        attributes: &'arena [AppliedAttribute<'arena>],
        flags: U8Flags<ConstantFlag>,
        constraint: SymbolConstraint<'arena>,
        ty: TypeSlot<'arena>,
        origin: Origin,
    ) -> Self {
        Self { span, name, attributes, flags, constraint, ty, origin }
    }
}

impl<'arena> Symbol<'arena> for ConstantSymbol<'arena> {
    fn path(&self) -> Path<'arena> {
        self.name
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn is_polyfill(&self) -> bool {
        self.flags.contains_bits(ConstantFlag::Polyfill as u8)
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }
}

impl ConstantSymbol<'_> {
    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.flags.contains_bits(ConstantFlag::Deprecated as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        self.flags.contains_bits(ConstantFlag::Internal as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_api(&self) -> bool {
        self.flags.contains_bits(ConstantFlag::API as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        self.flags.contains_bits(ConstantFlag::Experimental as u8)
    }
}

impl From<ConstantFlag> for u8 {
    #[inline]
    fn from(flag: ConstantFlag) -> Self {
        flag as u8
    }
}

impl HasSpan for ConstantSymbol<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
