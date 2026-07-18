use std::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::SymbolMember;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;
use crate::ty::Atom;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum EnumCaseFlag {
    Deprecated = 1 << 0,
    Backed = 1 << 1,
    Unit = 1 << 2,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct EnumCaseMember<'arena> {
    /// The span of the enum case.
    pub span: Span,
    /// The name of the enum case.
    pub name: Path<'arena>,
    /// The identifier of the symbol that defines this enum case.
    pub defining_symbol: SymbolId,
    /// The flags of the enum case.
    pub flags: U8Flags<EnumCaseFlag>,
    /// The constraint of the enum case, if any.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the enum case, if any.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The value of the enum case, if it is a backed enum case.
    pub value: Option<Atom<'arena>>,
    /// The origin of the enum case.
    pub origin: Origin,
}

impl<'arena> EnumCaseMember<'arena> {
    #[inline]
    #[must_use]
    pub const fn new(
        span: Span,
        name: Path<'arena>,
        defining_symbol: SymbolId,
        flags: U8Flags<EnumCaseFlag>,
        constraint: SymbolConstraint<'arena>,
        attributes: &'arena [AppliedAttribute<'arena>],
        value: Option<Atom<'arena>>,
        origin: Origin,
    ) -> Self {
        Self { span, name, defining_symbol, flags, constraint, attributes, value, origin }
    }
}

impl<'arena> SymbolMember<'arena> for EnumCaseMember<'arena> {
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

impl From<EnumCaseFlag> for u8 {
    #[inline]
    fn from(flag: EnumCaseFlag) -> Self {
        flag as u8
    }
}

impl HasSpan for EnumCaseMember<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

/// The cases of an enum: the members plus a SymbolId-sorted offset index for
/// O(log n) lookup by id.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct EnumCaseMemberList<'arena> {
    /// The cases in declaration order.
    pub members: &'arena [EnumCaseMember<'arena>],
    /// SymbolId-sorted offsets into `members`; O(log n) lookup by id.
    pub index: &'arena [u32],
}

impl<'arena> EnumCaseMemberList<'arena> {
    /// The cases in declaration order.
    #[inline]
    #[must_use]
    pub const fn members(&self) -> &'arena [EnumCaseMember<'arena>] {
        self.members
    }

    /// The number of cases in the list.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.members.len()
    }

    /// Whether the list has no cases.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Looks up a case by its identifier through the sorted index.
    #[must_use]
    pub fn get(&self, id: SymbolId) -> Option<&'arena EnumCaseMember<'arena>> {
        let members = self.members;
        let slot = self
            .index
            .binary_search_by(|&offset| {
                members.get(offset as usize).map_or(Ordering::Greater, |member| member.name.id.cmp(&id))
            })
            .ok()?;

        members.get(*self.index.get(slot)? as usize)
    }

    /// Whether a case with the given identifier exists in the list.
    #[inline]
    #[must_use]
    pub fn contains(&self, id: SymbolId) -> bool {
        self.get(id).is_some()
    }

    /// An iterator over the cases in declaration order.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'arena, EnumCaseMember<'arena>> {
        self.members.iter()
    }
}

impl<'arena> IntoIterator for &EnumCaseMemberList<'arena> {
    type Item = &'arena EnumCaseMember<'arena>;
    type IntoIter = std::slice::Iter<'arena, EnumCaseMember<'arena>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
