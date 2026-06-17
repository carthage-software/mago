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
pub enum ClassFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    Polyfill = 1 << 4,
    Final = 1 << 5,
    Abstract = 1 << 6,
    Readonly = 1 << 7,
    Immutable = 1 << 8,
    ConsistentConstructor = 1 << 9,
    ConsistentTemplates = 1 << 10,
    SealedProperties = 1 << 11,
    SealedMethods = 1 << 12,
    UnsealedProperties = 1 << 13,
    UnsealedMethods = 1 << 14,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ClassSymbol<'arena> {
    /// The span of the class symbol.
    pub span: Span,
    /// The origin of the symbol.
    pub origin: Origin,
    /// The name of the class symbol.
    pub name: Path<'arena>,
    /// The flags of the class symbol.
    pub flags: U16Flags<ClassFlag>,
    /// The constraint of the class symbol.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the class symbol.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The generic parameters of the class, if any.
    pub generics: &'arena [GenericParameter<'arena>],
    /// The transitive generic-parameter forwarding edges, sorted by
    /// `(parameter, target)` for binary-search lookup.
    pub forwardings: &'arena [GenericParameterForwarding<'arena>],
    /// The type aliases defined in the class symbol.
    pub aliases: TypeAliasMemberList<'arena>,
    /// The parent class the class extends, if any.
    pub extends: Option<InheritedType<'arena>>,
    /// The interfaces the class implements.
    pub implements: InheritedTypeList<'arena>,
    /// The traits the class uses.
    pub uses: InheritedTypeList<'arena>,
    /// The mixed-in class-likes whose members are pulled in (`@mixin`).
    pub mixins: InheritedTypeList<'arena>,
    /// The permitted inheritors of the class symbol (`@sealed`).
    pub permitted_inheritors: &'arena [InheritedType<'arena>],
    /// The sealed class-likes that list this class as a permitted inheritor,
    /// sorted by id for binary-search lookup.
    pub sealed_parents: &'arena [SymbolId],
    /// The constants of the class symbol.
    pub constants: ClassLikeConstantMemberList<'arena>,
    /// The properties of the class symbol.
    pub properties: PropertyMemberList<'arena>,
    /// The methods of the class symbol.
    pub methods: MethodMemberList<'arena>,
}

impl<'arena> Symbol<'arena> for ClassSymbol<'arena> {
    fn path(&self) -> Path<'arena> {
        self.name
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn is_polyfill(&self) -> bool {
        self.flags.contains_bits(ClassFlag::Polyfill as u16)
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }
}

impl ClassSymbol<'_> {
    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.flags.contains_bits(ClassFlag::Deprecated as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        self.flags.contains_bits(ClassFlag::Internal as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_api(&self) -> bool {
        self.flags.contains_bits(ClassFlag::API as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        self.flags.contains_bits(ClassFlag::Experimental as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_final(&self) -> bool {
        self.flags.contains_bits(ClassFlag::Final as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_abstract(&self) -> bool {
        self.flags.contains_bits(ClassFlag::Abstract as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        self.flags.contains_bits(ClassFlag::Readonly as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_immutable(&self) -> bool {
        self.flags.contains_bits(ClassFlag::Immutable as u16)
    }

    #[inline]
    #[must_use]
    pub const fn has_consistent_constructor(&self) -> bool {
        self.flags.contains_bits(ClassFlag::ConsistentConstructor as u16)
    }

    #[inline]
    #[must_use]
    pub const fn has_consistent_templates(&self) -> bool {
        self.flags.contains_bits(ClassFlag::ConsistentTemplates as u16)
    }
}

impl From<ClassFlag> for u16 {
    #[inline]
    fn from(flag: ClassFlag) -> Self {
        flag as u16
    }
}

impl HasSpan for ClassSymbol<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
