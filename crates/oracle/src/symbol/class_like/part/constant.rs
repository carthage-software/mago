use std::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::SymbolMember;
use crate::symbol::class_like::part::visibility::Visibility;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ClassLikeConstantFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    Final = 1 << 4,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ClassLikeConstantMember<'arena> {
    /// The span of the constant.
    pub span: Span,
    /// The visibility of the constant.
    pub visibility: Visibility,
    /// The name of the constant.
    pub name: Path<'arena>,
    /// The identifier of the symbol that defines this constant.
    pub defining_symbol: SymbolId,
    /// The flags of the constant.
    pub flags: U8Flags<ClassLikeConstantFlag>,
    /// The constraint of the constant, if any.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the constant, if any.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The type of the constant.
    pub ty: TypeSlot<'arena>,
    /// The origin of the constant.
    pub origin: Origin,
}

impl<'arena> ClassLikeConstantMember<'arena> {
    #[inline]
    #[must_use]
    pub const fn new(
        span: Span,
        visibility: Visibility,
        name: Path<'arena>,
        defining_symbol: SymbolId,
        flags: U8Flags<ClassLikeConstantFlag>,
        constraint: SymbolConstraint<'arena>,
        attributes: &'arena [AppliedAttribute<'arena>],
        ty: TypeSlot<'arena>,
        origin: Origin,
    ) -> Self {
        Self { span, visibility, name, defining_symbol, flags, constraint, attributes, ty, origin }
    }
}

impl<'arena> SymbolMember<'arena> for ClassLikeConstantMember<'arena> {
    fn path(&self) -> Path<'arena> {
        self.name
    }

    fn defining_symbol(&self) -> SymbolId {
        self.defining_symbol
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }
}

impl ClassLikeConstantMember<'_> {
    /// Returns true if the constant is deprecated.
    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.flags.contains_bits(ClassLikeConstantFlag::Deprecated as u8)
    }

    /// Returns true if the constant is internal.
    #[inline]
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        self.flags.contains_bits(ClassLikeConstantFlag::Internal as u8)
    }

    /// Returns true if the constant is part of the public API.
    #[inline]
    #[must_use]
    pub const fn is_api(&self) -> bool {
        self.flags.contains_bits(ClassLikeConstantFlag::API as u8)
    }

    /// Returns true if the constant is experimental.
    #[inline]
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        self.flags.contains_bits(ClassLikeConstantFlag::Experimental as u8)
    }

    /// Returns true if the constant is final.
    #[inline]
    #[must_use]
    pub const fn is_final(&self) -> bool {
        self.flags.contains_bits(ClassLikeConstantFlag::Final as u8)
    }
}

impl From<ClassLikeConstantFlag> for u8 {
    #[inline]
    fn from(flag: ClassLikeConstantFlag) -> Self {
        flag as u8
    }
}

impl HasSpan for ClassLikeConstantMember<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

/// The class constants of a class-like: the flattened members plus a
/// SymbolId-sorted offset index for O(log n) lookup by id.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ClassLikeConstantMemberList<'arena> {
    /// Flattened: own + inherited, source order.
    pub members: &'arena [ClassLikeConstantMember<'arena>],
    /// SymbolId-sorted offsets into `members`; O(log n) lookup by id.
    pub index: &'arena [u32],
}

impl<'arena> ClassLikeConstantMemberList<'arena> {
    /// The flattened members in source order.
    #[inline]
    #[must_use]
    pub const fn members(&self) -> &'arena [ClassLikeConstantMember<'arena>] {
        self.members
    }

    /// The number of members in the list.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.members.len()
    }

    /// Whether the list has no members.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Looks up a member by its identifier through the sorted index.
    #[must_use]
    pub fn get(&self, id: SymbolId) -> Option<&'arena ClassLikeConstantMember<'arena>> {
        let members = self.members;
        let slot = self
            .index
            .binary_search_by(|&offset| {
                members.get(offset as usize).map_or(Ordering::Greater, |member| member.name.id.cmp(&id))
            })
            .ok()?;

        members.get(*self.index.get(slot)? as usize)
    }

    /// Whether a member with the given identifier exists in the list.
    #[inline]
    #[must_use]
    pub fn contains(&self, id: SymbolId) -> bool {
        self.get(id).is_some()
    }

    /// An iterator over the members in source order.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'arena, ClassLikeConstantMember<'arena>> {
        self.members.iter()
    }
}

impl<'arena> IntoIterator for &ClassLikeConstantMemberList<'arena> {
    type Item = &'arena ClassLikeConstantMember<'arena>;
    type IntoIter = std::slice::Iter<'arena, ClassLikeConstantMember<'arena>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
