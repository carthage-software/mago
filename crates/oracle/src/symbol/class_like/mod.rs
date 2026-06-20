#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::class_like::anonymous_class::AnonymousClassSymbol;
use crate::symbol::class_like::class::ClassSymbol;
use crate::symbol::class_like::r#enum::EnumSymbol;
use crate::symbol::class_like::interface::InterfaceSymbol;
use crate::symbol::class_like::part::alias::TypeAliasMemberList;
use crate::symbol::class_like::part::constant::ClassLikeConstantMemberList;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::class_like::part::inheritance::InheritedTypeList;
use crate::symbol::class_like::part::method::MethodMemberList;
use crate::symbol::class_like::part::property::PropertyMemberList;
use crate::symbol::class_like::r#trait::TraitSymbol;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::generic::GenericParameter;
use crate::symbol::part::generic::GenericParameterForwarding;
use crate::symbol::part::origin::Origin;

pub mod anonymous_class;
pub mod class;
pub mod r#enum;
pub mod interface;
pub mod part;
pub mod r#trait;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ClassLikeKind {
    Class,
    Interface,
    Enum,
    Trait,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum ClassLikeSymbol<'arena> {
    Class(&'arena ClassSymbol<'arena>),
    Interface(&'arena InterfaceSymbol<'arena>),
    Trait(&'arena TraitSymbol<'arena>),
    Enum(&'arena EnumSymbol<'arena>),
    AnonymousClass(&'arena AnonymousClassSymbol<'arena>),
}

impl<'arena> ClassLikeSymbol<'arena> {
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> ClassLikeKind {
        match self {
            ClassLikeSymbol::Class(_) => ClassLikeKind::Class,
            ClassLikeSymbol::Interface(_) => ClassLikeKind::Interface,
            ClassLikeSymbol::Trait(_) => ClassLikeKind::Trait,
            ClassLikeSymbol::Enum(_) => ClassLikeKind::Enum,
            ClassLikeSymbol::AnonymousClass(_) => ClassLikeKind::Class,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_class(&self) -> bool {
        matches!(self, ClassLikeSymbol::Class(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_interface(&self) -> bool {
        matches!(self, ClassLikeSymbol::Interface(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_trait(&self) -> bool {
        matches!(self, ClassLikeSymbol::Trait(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_enum(&self) -> bool {
        matches!(self, ClassLikeSymbol::Enum(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_anonymous_class(&self) -> bool {
        matches!(self, ClassLikeSymbol::AnonymousClass(_))
    }

    #[inline]
    #[must_use]
    pub const fn generics(&self) -> &'arena [GenericParameter<'arena>] {
        match self {
            ClassLikeSymbol::Class(symbol) => symbol.generics,
            ClassLikeSymbol::Interface(symbol) => symbol.generics,
            ClassLikeSymbol::Trait(symbol) => symbol.generics,
            _ => &[],
        }
    }

    /// The constants declared on the class-like.
    #[inline]
    #[must_use]
    pub const fn constants(&self) -> ClassLikeConstantMemberList<'arena> {
        match self {
            ClassLikeSymbol::Class(symbol) => symbol.constants,
            ClassLikeSymbol::Interface(symbol) => symbol.constants,
            ClassLikeSymbol::Trait(symbol) => symbol.constants,
            ClassLikeSymbol::Enum(symbol) => symbol.constants,
            ClassLikeSymbol::AnonymousClass(symbol) => symbol.constants,
        }
    }

    /// The methods declared on the class-like.
    #[inline]
    #[must_use]
    pub const fn methods(&self) -> MethodMemberList<'arena> {
        match self {
            ClassLikeSymbol::Class(symbol) => symbol.methods,
            ClassLikeSymbol::Interface(symbol) => symbol.methods,
            ClassLikeSymbol::Trait(symbol) => symbol.methods,
            ClassLikeSymbol::Enum(symbol) => symbol.methods,
            ClassLikeSymbol::AnonymousClass(symbol) => symbol.methods,
        }
    }

    /// The properties declared on the class-like, or `None` for an enum (which cannot declare properties).
    #[inline]
    #[must_use]
    pub const fn properties(&self) -> Option<PropertyMemberList<'arena>> {
        match self {
            ClassLikeSymbol::Class(symbol) => Some(symbol.properties),
            ClassLikeSymbol::Interface(symbol) => Some(symbol.properties),
            ClassLikeSymbol::Trait(symbol) => Some(symbol.properties),
            ClassLikeSymbol::AnonymousClass(symbol) => Some(symbol.properties),
            ClassLikeSymbol::Enum(_) => None,
        }
    }

    /// The generic-parameter forwardings declared on the class-like.
    #[inline]
    #[must_use]
    pub const fn forwardings(&self) -> &'arena [GenericParameterForwarding<'arena>] {
        match self {
            ClassLikeSymbol::Class(class) => class.forwardings,
            ClassLikeSymbol::Interface(interface) => interface.forwardings,
            ClassLikeSymbol::Trait(r#trait) => r#trait.forwardings,
            ClassLikeSymbol::Enum(_) | ClassLikeSymbol::AnonymousClass(_) => &[],
        }
    }

    /// The traits used by the class-like.
    #[inline]
    #[must_use]
    pub const fn uses(&self) -> InheritedTypeList<'arena> {
        match self {
            ClassLikeSymbol::Class(class) => class.uses,
            ClassLikeSymbol::Trait(r#trait) => r#trait.uses,
            ClassLikeSymbol::Enum(r#enum) => r#enum.uses,
            ClassLikeSymbol::AnonymousClass(anonymous_class) => anonymous_class.uses,
            ClassLikeSymbol::Interface(_) => InheritedTypeList { edges: &[], index: &[] },
        }
    }

    /// The type aliases declared on the class-like.
    #[inline]
    #[must_use]
    pub const fn aliases(&self) -> TypeAliasMemberList<'arena> {
        match self {
            ClassLikeSymbol::Class(class) => class.aliases,
            ClassLikeSymbol::Interface(interface) => interface.aliases,
            ClassLikeSymbol::Trait(r#trait) => r#trait.aliases,
            ClassLikeSymbol::Enum(r#enum) => r#enum.aliases,
            ClassLikeSymbol::AnonymousClass(_) => TypeAliasMemberList { members: &[], index: &[] },
        }
    }

    /// The inheritance edge from this class-like to `ancestor`, searched across its
    /// superclass, interfaces, and used traits without allocating. The list lookups
    /// binary-search their `target.id` index.
    #[inline]
    #[must_use]
    pub fn inheritance_edge_to(&self, ancestor: SymbolId) -> Option<InheritedType<'arena>> {
        let superclass = match self {
            ClassLikeSymbol::Class(class) => class.extends,
            ClassLikeSymbol::AnonymousClass(anonymous_class) => anonymous_class.extends,
            _ => None,
        };
        if let Some(extends) = superclass {
            if extends.target.id == ancestor {
                return Some(extends);
            }
        }

        let lists: [InheritedTypeList<'arena>; 2] = match self {
            ClassLikeSymbol::Class(class) => [class.implements, class.uses],
            ClassLikeSymbol::Interface(interface) => [interface.extends, InheritedTypeList { edges: &[], index: &[] }],
            ClassLikeSymbol::Trait(r#trait) => [r#trait.uses, InheritedTypeList { edges: &[], index: &[] }],
            ClassLikeSymbol::Enum(r#enum) => [r#enum.implements, r#enum.uses],
            ClassLikeSymbol::AnonymousClass(anonymous_class) => [anonymous_class.implements, anonymous_class.uses],
        };

        lists.iter().find_map(|list| list.get(ancestor).copied())
    }

    /// The sealed parents of the class-like.
    #[inline]
    #[must_use]
    pub const fn sealed_parents(&self) -> &'arena [SymbolId] {
        match self {
            ClassLikeSymbol::Class(class) => class.sealed_parents,
            ClassLikeSymbol::Interface(interface) => interface.sealed_parents,
            ClassLikeSymbol::Enum(r#enum) => r#enum.sealed_parents,
            ClassLikeSymbol::AnonymousClass(anonymous_class) => anonymous_class.sealed_parents,
            ClassLikeSymbol::Trait(_) => &[],
        }
    }

    /// This class-like's direct inheritance edges: its (optional) superclass
    /// followed by the two edge lists that carry the rest (implements/uses, or
    /// an interface's parent interfaces). Used to walk the inheritance chain
    /// when resolving a transitively-inherited type argument.
    #[inline]
    #[must_use]
    pub fn inheritance_edges(&self) -> (Option<InheritedType<'arena>>, [InheritedTypeList<'arena>; 2]) {
        let empty = InheritedTypeList { edges: &[], index: &[] };
        match self {
            ClassLikeSymbol::Class(class) => (class.extends, [class.implements, class.uses]),
            ClassLikeSymbol::Interface(interface) => (None, [interface.extends, empty]),
            ClassLikeSymbol::Trait(r#trait) => (None, [r#trait.uses, empty]),
            ClassLikeSymbol::Enum(r#enum) => (None, [r#enum.implements, r#enum.uses]),
            ClassLikeSymbol::AnonymousClass(anonymous_class) => {
                (anonymous_class.extends, [anonymous_class.implements, anonymous_class.uses])
            }
        }
    }

    /// The permitted inheritors of the class-like, or `None` if it cannot seal its inheritors.
    #[inline]
    #[must_use]
    pub const fn permitted_inheritors(&self) -> Option<&'arena [InheritedType<'arena>]> {
        Some(match self {
            ClassLikeSymbol::Class(class) => class.permitted_inheritors,
            ClassLikeSymbol::Interface(interface) => interface.permitted_inheritors,
            _ => return None,
        })
    }
}

impl<'arena> Symbol<'arena> for ClassLikeSymbol<'arena> {
    fn path(&self) -> Path<'arena> {
        match self {
            ClassLikeSymbol::Class(symbol) => symbol.path(),
            ClassLikeSymbol::Interface(symbol) => symbol.path(),
            ClassLikeSymbol::Trait(symbol) => symbol.path(),
            ClassLikeSymbol::Enum(symbol) => symbol.path(),
            ClassLikeSymbol::AnonymousClass(symbol) => symbol.path(),
        }
    }

    fn origin(&self) -> Origin {
        match self {
            ClassLikeSymbol::Class(symbol) => symbol.origin(),
            ClassLikeSymbol::Interface(symbol) => symbol.origin(),
            ClassLikeSymbol::Trait(symbol) => symbol.origin(),
            ClassLikeSymbol::Enum(symbol) => symbol.origin(),
            ClassLikeSymbol::AnonymousClass(symbol) => symbol.origin(),
        }
    }

    fn is_polyfill(&self) -> bool {
        match self {
            ClassLikeSymbol::Class(symbol) => symbol.is_polyfill(),
            ClassLikeSymbol::Interface(symbol) => symbol.is_polyfill(),
            ClassLikeSymbol::Trait(symbol) => symbol.is_polyfill(),
            ClassLikeSymbol::Enum(symbol) => symbol.is_polyfill(),
            ClassLikeSymbol::AnonymousClass(symbol) => symbol.is_polyfill(),
        }
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        match self {
            ClassLikeSymbol::Class(symbol) => symbol.constraint(),
            ClassLikeSymbol::Interface(symbol) => symbol.constraint(),
            ClassLikeSymbol::Trait(symbol) => symbol.constraint(),
            ClassLikeSymbol::Enum(symbol) => symbol.constraint(),
            ClassLikeSymbol::AnonymousClass(symbol) => symbol.constraint(),
        }
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        match self {
            ClassLikeSymbol::Class(symbol) => symbol.applied_attributes(),
            ClassLikeSymbol::Interface(symbol) => symbol.applied_attributes(),
            ClassLikeSymbol::Trait(symbol) => symbol.applied_attributes(),
            ClassLikeSymbol::Enum(symbol) => symbol.applied_attributes(),
            ClassLikeSymbol::AnonymousClass(symbol) => symbol.applied_attributes(),
        }
    }
}

impl HasSpan for ClassLikeSymbol<'_> {
    fn span(&self) -> Span {
        match self {
            ClassLikeSymbol::Class(symbol) => symbol.span(),
            ClassLikeSymbol::Interface(symbol) => symbol.span(),
            ClassLikeSymbol::Trait(symbol) => symbol.span(),
            ClassLikeSymbol::Enum(symbol) => symbol.span(),
            ClassLikeSymbol::AnonymousClass(symbol) => symbol.span(),
        }
    }
}
