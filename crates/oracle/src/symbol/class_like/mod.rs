#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::class_like::anonymous_class::AnonymousClassSymbol;
use crate::symbol::class_like::class::ClassSymbol;
use crate::symbol::class_like::r#enum::EnumSymbol;
use crate::symbol::class_like::interface::InterfaceSymbol;
use crate::symbol::class_like::part::constant::ClassLikeConstantMemberList;
use crate::symbol::class_like::part::method::MethodMemberList;
use crate::symbol::class_like::part::property::PropertyMemberList;
use crate::symbol::class_like::r#trait::TraitSymbol;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;

pub mod anonymous_class;
pub mod class;
pub mod r#enum;
pub mod interface;
pub mod part;
pub mod r#trait;

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
