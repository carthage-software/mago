use std::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::SymbolMember;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct TypeAliasMember<'arena> {
    /// The span of the type alias symbol.
    pub span: Span,
    /// The name of the type alias symbol.
    pub name: Path<'arena>,
    /// Where the type alias symbol is defined.
    pub defining_symbol: SymbolId,
    /// The type of the type alias symbol.
    pub ty: TypeSlot<'arena>,
    /// The origin of the type alias symbol.
    pub origin: Origin,
}

impl<'arena> SymbolMember<'arena> for TypeAliasMember<'arena> {
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
        &[]
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        None
    }
}

impl HasSpan for TypeAliasMember<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

/// The type aliases of a class-like: the members plus a SymbolId-sorted offset
/// index for O(log n) lookup by id.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct TypeAliasMemberList<'arena> {
    /// Flattened: own + inherited, source order.
    pub members: &'arena [TypeAliasMember<'arena>],
    /// SymbolId-sorted offsets into `members`; O(log n) lookup by id.
    pub index: &'arena [u32],
}

impl<'arena> TypeAliasMemberList<'arena> {
    /// The members in source order.
    #[inline]
    #[must_use]
    pub const fn members(&self) -> &'arena [TypeAliasMember<'arena>] {
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
    pub fn get(&self, id: SymbolId) -> Option<&'arena TypeAliasMember<'arena>> {
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
    pub fn iter(&self) -> std::slice::Iter<'arena, TypeAliasMember<'arena>> {
        self.members.iter()
    }
}

impl<'arena> IntoIterator for &TypeAliasMemberList<'arena> {
    type Item = &'arena TypeAliasMember<'arena>;
    type IntoIter = std::slice::Iter<'arena, TypeAliasMember<'arena>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
