#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::class_like::part::alias::TypeAliasMemberList;
use crate::symbol::class_like::part::constant::ClassLikeConstantMemberList;
use crate::symbol::class_like::part::enum_case::EnumCaseMemberList;
use crate::symbol::class_like::part::inheritance::InheritedTypeList;
use crate::symbol::class_like::part::method::MethodMemberList;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum EnumFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    Polyfill = 1 << 4,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum EnumBackingType {
    Int,
    String,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct EnumSymbol<'arena> {
    /// The span of the enum symbol.
    pub span: Span,
    /// The origin of the symbol.
    pub origin: Origin,
    /// The name of the enum symbol.
    pub name: Path<'arena>,
    /// The flags of the enum symbol.
    pub flags: U8Flags<EnumFlag>,
    /// The backing type of the enum symbol, if any.
    pub backing_type: Option<EnumBackingType>,
    /// The constraint of the enum symbol.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the enum symbol.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The type aliases defined in the enum symbol.
    pub aliases: TypeAliasMemberList<'arena>,
    /// The implemented interfaces of the enum symbol.
    pub implements: InheritedTypeList<'arena>,
    /// The traits used by the enum symbol.
    pub uses: InheritedTypeList<'arena>,
    /// The sealed class-likes that list this enum as a permitted inheritor,
    /// sorted by id for binary-search lookup.
    pub sealed_parents: &'arena [SymbolId],
    /// The constants of the enum symbol.
    pub constants: ClassLikeConstantMemberList<'arena>,
    /// The cases of the enum symbol.
    pub cases: EnumCaseMemberList<'arena>,
    /// The methods of the enum symbol.
    pub methods: MethodMemberList<'arena>,
}

impl<'arena> Symbol<'arena> for EnumSymbol<'arena> {
    fn path(&self) -> Path<'arena> {
        self.name
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn is_polyfill(&self) -> bool {
        self.flags.contains_bits(EnumFlag::Polyfill as u8)
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }
}

impl EnumSymbol<'_> {
    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.flags.contains_bits(EnumFlag::Deprecated as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        self.flags.contains_bits(EnumFlag::Internal as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_api(&self) -> bool {
        self.flags.contains_bits(EnumFlag::API as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        self.flags.contains_bits(EnumFlag::Experimental as u8)
    }
}

impl HasSpan for EnumSymbol<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl From<EnumFlag> for u8 {
    #[inline]
    fn from(flag: EnumFlag) -> Self {
        flag as u8
    }
}
