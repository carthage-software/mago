#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U16Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::class_like::part::alias::TypeAliasMemberList;
use crate::symbol::class_like::part::constant::ClassLikeConstantMemberList;
use crate::symbol::class_like::part::inheritance::InheritedType;
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
pub enum InterfaceFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    Polyfill = 1 << 4,
    SealedProperties = 1 << 5,
    SealedMethods = 1 << 6,
    UnsealedProperties = 1 << 7,
    UnsealedMethods = 1 << 8,
    EnumInterface = 1 << 9,
    Immutable = 1 << 10,
    ConsistentConstructor = 1 << 11,
    ConsistentTemplates = 1 << 12,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct InterfaceSymbol<'arena> {
    /// The span of the interface symbol.
    pub span: Span,
    /// The origin of the symbol.
    pub origin: Origin,
    /// The name of the interface symbol.
    pub name: Path<'arena>,
    /// The flags of the interface symbol.
    pub flags: U16Flags<InterfaceFlag>,
    /// The constraint of the interface symbol.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the interface symbol.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The generic parameters of the interface, if any.
    pub generics: &'arena [GenericParameter<'arena>],
    /// The transitive generic-parameter forwarding edges, sorted by
    /// `(parameter, target)` for binary-search lookup.
    pub forwardings: &'arena [GenericParameterForwarding<'arena>],
    /// The type aliases defined in the interface symbol.
    pub aliases: TypeAliasMemberList<'arena>,
    /// The extended interfaces of the interface symbol.
    pub extends: InheritedTypeList<'arena>,
    /// The classes an implementor is required to extend (`@require-extends`).
    pub require_extends: InheritedTypeList<'arena>,
    /// The interfaces an implementor is required to implement (`@require-implements`).
    pub require_implements: InheritedTypeList<'arena>,
    /// The mixed-in class-likes whose members are pulled in (`@mixin`).
    pub mixins: InheritedTypeList<'arena>,
    /// The permitted inheritors of the interface symbol.
    pub permitted_inheritors: &'arena [InheritedType<'arena>],
    /// The sealed class-likes that list this interface as a permitted inheritor,
    /// sorted by id for binary-search lookup.
    pub sealed_parents: &'arena [SymbolId],
    /// The constants of the interface symbol.
    pub constants: ClassLikeConstantMemberList<'arena>,
    /// The properties of the interface symbol.
    pub properties: PropertyMemberList<'arena>,
    /// The methods of the interface symbol.
    pub methods: MethodMemberList<'arena>,
}

impl<'arena> Symbol<'arena> for InterfaceSymbol<'arena> {
    fn path(&self) -> Path<'arena> {
        self.name
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn is_polyfill(&self) -> bool {
        self.flags.contains_bits(InterfaceFlag::Polyfill as u16)
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }
}

impl InterfaceSymbol<'_> {
    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.flags.contains_bits(InterfaceFlag::Deprecated as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        self.flags.contains_bits(InterfaceFlag::Internal as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_api(&self) -> bool {
        self.flags.contains_bits(InterfaceFlag::API as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        self.flags.contains_bits(InterfaceFlag::Experimental as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_immutable(&self) -> bool {
        self.flags.contains_bits(InterfaceFlag::Immutable as u16)
    }

    #[inline]
    #[must_use]
    pub const fn has_consistent_constructor(&self) -> bool {
        self.flags.contains_bits(InterfaceFlag::ConsistentConstructor as u16)
    }

    #[inline]
    #[must_use]
    pub const fn has_consistent_templates(&self) -> bool {
        self.flags.contains_bits(InterfaceFlag::ConsistentTemplates as u16)
    }
}

impl HasSpan for InterfaceSymbol<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl From<InterfaceFlag> for u16 {
    #[inline]
    fn from(flag: InterfaceFlag) -> Self {
        flag as u16
    }
}
