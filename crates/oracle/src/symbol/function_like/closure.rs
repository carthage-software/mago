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
pub enum ClosureFlag {
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
pub struct ClosureSymbol<'arena> {
    /// The span of the closure symbol.
    pub span: Span,
    /// The synthesized name of the closure symbol.
    pub name: Path<'arena>,
    /// The flags of the closure symbol.
    pub flags: U16Flags<ClosureFlag>,
    /// The constraint of the closure symbol.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the closure symbol.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The generic parameters of the closure symbol.
    pub generics: &'arena [GenericParameter<'arena>],
    /// The parameters of the closure symbol.
    pub params: &'arena [SignatureParameter<'arena>],
    /// The parameters that determine if the closure is pure.
    pub pure_unless_impure_params: &'arena [u32],
    /// The return type of the closure symbol.
    pub ret: TypeSlot<'arena>,
    /// The types that the closure symbol can throw.
    pub throws: &'arena [Type<'arena>],
    /// The assertions that the closure makes about its parameters.
    pub assertions: &'arena [FunctionLikeAssertion<'arena>],
    /// The global variables that the closure symbol accesses.
    pub accessed_globals: &'arena [Var<'arena>],
    /// The origin of the symbol.
    pub origin: Origin,
}

impl<'arena> Symbol<'arena> for ClosureSymbol<'arena> {
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

impl From<ClosureFlag> for u16 {
    #[inline]
    fn from(flag: ClosureFlag) -> Self {
        flag as u16
    }
}

impl HasSpan for ClosureSymbol<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
