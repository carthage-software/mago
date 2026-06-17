#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U16Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::function_like::part::assertion::FunctionLikeAssertion;
use crate::symbol::function_like::part::parameter::SignatureParameter;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::generic::GenericParameter;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;
use crate::ty::Type;
use crate::var::Var;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u16)]
pub enum FunctionFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    HasYield = 1 << 4,
    HasThrow = 1 << 5,
    MustUse = 1 << 6,
    Pure = 1 << 7,
    IgnoreNullableReturn = 1 << 8,
    IgnoreFalsableReturn = 1 << 9,
    NoNamedArguments = 1 << 10,
    ReturnsByReference = 1 << 11,
    SuspendsFiber = 1 << 12,
    Polyfill = 1 << 13,
    AssertionsInferred = 1 << 14,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct FunctionSymbol<'arena> {
    /// The span of the function symbol.
    pub span: Span,
    /// The fully qualified name of the function symbol.
    pub name: Path<'arena>,
    /// The flags of the function symbol.
    pub flags: U16Flags<FunctionFlag>,
    /// The constraint of the function symbol.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the function symbol.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The generic parameters of the function symbol.
    pub generics: &'arena [GenericParameter<'arena>],
    /// The parameters of the function symbol.
    pub params: &'arena [SignatureParameter<'arena>],
    /// The return type of the function symbol.
    pub ret: TypeSlot<'arena>,
    /// The types that the function symbol can throw.
    pub throws: &'arena [Type<'arena>],
    /// The assertions that the function makes about its parameters.
    pub assertions: &'arena [FunctionLikeAssertion<'arena>],
    /// The global variables that the function symbol accesses.
    pub accessed_globals: &'arena [Var<'arena>],
    /// The origin of the symbol.
    pub origin: Origin,
}

impl<'arena> Symbol<'arena> for FunctionSymbol<'arena> {
    fn path(&self) -> Path<'arena> {
        self.name
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn is_polyfill(&self) -> bool {
        self.flags.contains_bits(FunctionFlag::Polyfill as u16)
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }
}

impl FunctionSymbol<'_> {
    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.flags.contains_bits(FunctionFlag::Deprecated as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        self.flags.contains_bits(FunctionFlag::Internal as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_api(&self) -> bool {
        self.flags.contains_bits(FunctionFlag::API as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        self.flags.contains_bits(FunctionFlag::Experimental as u16)
    }
}

impl From<FunctionFlag> for u16 {
    #[inline]
    fn from(flag: FunctionFlag) -> Self {
        flag as u16
    }
}

impl HasSpan for FunctionSymbol<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
