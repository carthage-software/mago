#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::SymbolMember;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum SignatureParameterFlag {
    Variadic = 1 << 0,
    ByReference = 1 << 1,
    Promoted = 1 << 2,
    HasDefault = 1 << 3,
    Nullable = 1 << 4,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct SignatureParameter<'arena> {
    pub span: Span,
    pub defining_symbol: SymbolId,
    pub path: Path<'arena>,
    pub attributes: &'arena [AppliedAttribute<'arena>],
    pub flags: U8Flags<SignatureParameterFlag>,
    pub constraint: SymbolConstraint<'arena>,
    pub ty: TypeSlot<'arena>,
    pub out_ty: TypeSlot<'arena>,
    pub default_ty: TypeSlot<'arena>,
    pub origin: Origin,
}

impl<'arena> SymbolMember<'arena> for SignatureParameter<'arena> {
    fn path(&self) -> Path<'arena> {
        self.path
    }

    fn defining_symbol(&self) -> SymbolId {
        self.defining_symbol
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }
}

impl SignatureParameter<'_> {
    #[inline]
    #[must_use]
    pub const fn is_variadic(&self) -> bool {
        self.flags.contains_bits(SignatureParameterFlag::Variadic as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_by_reference(&self) -> bool {
        self.flags.contains_bits(SignatureParameterFlag::ByReference as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_promoted(&self) -> bool {
        self.flags.contains_bits(SignatureParameterFlag::Promoted as u8)
    }

    #[inline]
    #[must_use]
    pub const fn has_default(&self) -> bool {
        self.flags.contains_bits(SignatureParameterFlag::HasDefault as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_nullable(&self) -> bool {
        self.flags.contains_bits(SignatureParameterFlag::Nullable as u8)
    }
}

impl From<SignatureParameterFlag> for u8 {
    #[inline]
    fn from(flag: SignatureParameterFlag) -> Self {
        flag as u8
    }
}

impl HasSpan for SignatureParameter<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
