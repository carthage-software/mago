#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U16Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::class_like::part::alias::TypeAliasMemberList;
use crate::symbol::class_like::part::constant::ClassLikeConstantMemberList;
use crate::symbol::class_like::part::inheritance::InheritedTypeList;
use crate::symbol::class_like::part::method::MethodMemberList;
use crate::symbol::class_like::part::property::PropertyMemberList;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::generic::GenericParameter;
use crate::symbol::part::generic::GenericParameterForwarding;
use crate::symbol::part::origin::Origin;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u16)]
pub enum TraitFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    Polyfill = 1 << 4,
    Immutable = 1 << 5,
    ConsistentConstructor = 1 << 6,
    ConsistentTemplates = 1 << 7,
    SealedProperties = 1 << 8,
    SealedMethods = 1 << 9,
    UnsealedProperties = 1 << 10,
    UnsealedMethods = 1 << 11,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct TraitSymbol<'arena> {
    /// The span of the trait symbol.
    pub span: Span,
    /// The origin of the symbol.
    pub origin: Origin,
    /// The name of the trait symbol.
    pub name: Path<'arena>,
    /// The flags of the trait symbol.
    pub flags: U16Flags<TraitFlag>,
    /// The constraint of the trait symbol.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the trait symbol.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The generic parameters of the trait, if any.
    pub generics: &'arena [GenericParameter<'arena>],
    /// The transitive generic-parameter forwarding edges, sorted by
    /// `(parameter, target)` for binary-search lookup.
    pub forwardings: &'arena [GenericParameterForwarding<'arena>],
    /// The type aliases defined in the trait symbol.
    pub aliases: TypeAliasMemberList<'arena>,
    /// The traits the trait uses.
    pub uses: InheritedTypeList<'arena>,
    /// The classes a using class is required to extend (`@require-extends`).
    pub require_extends: InheritedTypeList<'arena>,
    /// The interfaces a using class is required to implement (`@require-implements`).
    pub require_implements: InheritedTypeList<'arena>,
    /// The mixed-in class-likes whose members are pulled in (`@mixin`).
    pub mixins: InheritedTypeList<'arena>,
    /// The constants of the trait symbol.
    pub constants: ClassLikeConstantMemberList<'arena>,
    /// The properties of the trait symbol.
    pub properties: PropertyMemberList<'arena>,
    /// The methods of the trait symbol.
    pub methods: MethodMemberList<'arena>,
}

impl<'arena> Symbol<'arena> for TraitSymbol<'arena> {
    fn path(&self) -> Path<'arena> {
        self.name
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn is_polyfill(&self) -> bool {
        self.flags.contains_bits(TraitFlag::Polyfill as u16)
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }
}

impl TraitSymbol<'_> {
    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.flags.contains_bits(TraitFlag::Deprecated as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        self.flags.contains_bits(TraitFlag::Internal as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_api(&self) -> bool {
        self.flags.contains_bits(TraitFlag::API as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        self.flags.contains_bits(TraitFlag::Experimental as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_immutable(&self) -> bool {
        self.flags.contains_bits(TraitFlag::Immutable as u16)
    }

    #[inline]
    #[must_use]
    pub const fn has_consistent_constructor(&self) -> bool {
        self.flags.contains_bits(TraitFlag::ConsistentConstructor as u16)
    }

    #[inline]
    #[must_use]
    pub const fn has_consistent_templates(&self) -> bool {
        self.flags.contains_bits(TraitFlag::ConsistentTemplates as u16)
    }
}

impl From<TraitFlag> for u16 {
    #[inline]
    fn from(flag: TraitFlag) -> Self {
        flag as u16
    }
}

impl HasSpan for TraitSymbol<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
