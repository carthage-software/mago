#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::class_like::part::constant::ClassLikeConstantMemberList;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::class_like::part::inheritance::InheritedTypeList;
use crate::symbol::class_like::part::method::MethodMemberList;
use crate::symbol::class_like::part::property::PropertyMemberList;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum AnonymousClassFlag {
    Readonly = 1 << 0,
    Immutable = 1 << 1,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct AnonymousClassSymbol<'arena> {
    /// The span of the anonymous class symbol.
    pub span: Span,
    /// The origin of the symbol.
    pub origin: Origin,
    /// The synthesized name of the anonymous class symbol.
    pub name: Path<'arena>,
    /// The flags of the anonymous class symbol.
    pub flags: U8Flags<AnonymousClassFlag>,
    /// The constraint of the anonymous class symbol.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the anonymous class symbol.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The parent class the anonymous class extends, if any.
    pub extends: Option<InheritedType<'arena>>,
    /// The interfaces the anonymous class implements.
    pub implements: InheritedTypeList<'arena>,
    /// The traits the anonymous class uses.
    pub uses: InheritedTypeList<'arena>,
    /// The mixed-in class-likes whose members are pulled in (`@mixin`).
    pub mixins: InheritedTypeList<'arena>,
    /// The sealed class-likes that list this anonymous class as a permitted
    /// inheritor, sorted by id for binary-search lookup.
    pub sealed_parents: &'arena [SymbolId],
    /// The constants of the anonymous class symbol.
    pub constants: ClassLikeConstantMemberList<'arena>,
    /// The properties of the anonymous class symbol.
    pub properties: PropertyMemberList<'arena>,
    /// The methods of the anonymous class symbol.
    pub methods: MethodMemberList<'arena>,
}

impl<'arena> Symbol<'arena> for AnonymousClassSymbol<'arena> {
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

impl AnonymousClassSymbol<'_> {
    #[inline]
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        self.flags.contains_bits(AnonymousClassFlag::Readonly as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_immutable(&self) -> bool {
        self.flags.contains_bits(AnonymousClassFlag::Immutable as u8)
    }
}

impl From<AnonymousClassFlag> for u8 {
    #[inline]
    fn from(flag: AnonymousClassFlag) -> Self {
        flag as u8
    }
}

impl HasSpan for AnonymousClassSymbol<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
