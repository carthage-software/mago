#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::ty::Type;

/// Variance of a generic parameter.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Variance {
    /// The generic parameter is invariant.
    ///
    /// This means that the generic parameter can only be used in the exact type it is defined in.
    #[default]
    Invariant,
    /// The generic parameter is covariant.
    ///
    /// This means that the generic parameter can be used in a subtype of the type it is defined in.
    Covariant,
    /// The generic parameter is contravariant.
    ///
    /// This means that the generic parameter can be used in a supertype of the type it is defined in.
    Contravariant,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GenericParameter<'arena> {
    /// The span of the generic parameter.
    pub span: Span,
    /// The name of the generic parameter.
    pub name: &'arena [u8],
    /// The entity that defines the generic parameter.
    pub defining_entity: SymbolId,
    /// The variance of the generic parameter.
    pub variance: Variance,
    /// The constraint of the generic parameter.
    pub constraint: Type<'arena>,
    /// The default type of the generic parameter.
    pub default: Option<Type<'arena>>,
}

/// Identifies one generic parameter: the entity that declares it and the
/// parameter's name.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GenericParameterId<'arena> {
    /// The entity that defines the generic parameter.
    pub defining_entity: SymbolId,
    /// The name of the generic parameter.
    pub name: &'arena [u8],
}

/// A resolved forwarding edge: a class binds its own `parameter` into
/// `target`'s slot. The linker stores the transitive closure, so a single
/// edge captures forwarding through any depth of inheritance.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GenericParameterForwarding<'arena> {
    /// The forwarding entity's own generic parameter.
    pub parameter: &'arena [u8],
    /// The generic parameter the forwarding entity binds it into.
    pub target: GenericParameterId<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WhereConstraint<'arena> {
    /// The span of the where constraint
    pub span: Span,
    /// The name of the generic parameter that the constraint applies to.
    pub parameter: &'arena [u8],
    /// The type that the generic parameter is constrained to.
    pub ty: Type<'arena>,
}

impl HasSpan for GenericParameter<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for WhereConstraint<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
