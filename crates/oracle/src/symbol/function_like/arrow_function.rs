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

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u16)]
pub enum ArrowFunctionFlag {
    HasYield = 1 << 1,
    HasThrow = 1 << 2,
    MustUse = 1 << 3,
    Pure = 1 << 4,
    IgnoreNullableReturn = 1 << 5,
    IgnoreFalsableReturn = 1 << 6,
    NoNamedArguments = 1 << 7,
    ReturnsByReference = 1 << 8,
    SuspendsFiber = 1 << 9,
    AssertionsInferred = 1 << 10,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ArrowFunctionSymbol<'arena> {
    /// The span of the ArrowFunction symbol.
    pub span: Span,
    /// The synthesized name of the ArrowFunction symbol.
    pub name: Path<'arena>,
    /// The flags of the ArrowFunction symbol.
    pub flags: U16Flags<ArrowFunctionFlag>,
    /// The constraint of the ArrowFunction symbol.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the ArrowFunction symbol.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The generic parameters of the ArrowFunction symbol.
    pub generics: &'arena [GenericParameter<'arena>],
    /// The parameters of the ArrowFunction symbol.
    pub params: &'arena [SignatureParameter<'arena>],
    /// The return type of the ArrowFunction symbol.
    pub ret: TypeSlot<'arena>,
    /// The types that the ArrowFunction symbol can throw.
    pub throws: &'arena [Type<'arena>],
    /// The assertions that the ArrowFunction makes about its parameters.
    pub assertions: &'arena [FunctionLikeAssertion<'arena>],
    /// The origin of the symbol.
    pub origin: Origin,
}

impl<'arena> Symbol<'arena> for ArrowFunctionSymbol<'arena> {
    fn path(&self) -> Path<'arena> {
        self.name
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn is_polyfill(&self) -> bool {
        false
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }
}

impl From<ArrowFunctionFlag> for u16 {
    #[inline]
    fn from(flag: ArrowFunctionFlag) -> Self {
        flag as u16
    }
}

impl HasSpan for ArrowFunctionSymbol<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
