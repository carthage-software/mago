use std::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U16Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::SymbolMember;
use crate::symbol::class_like::part::property_hook::PropertyHookMember;
use crate::symbol::class_like::part::visibility::ReadWriteVisibility;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u16)]
pub enum PropertyFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    Static = 1 << 4,
    Readonly = 1 << 5,
    Final = 1 << 6,
    Abstract = 1 << 7,
    Promoted = 1 << 8,
    Virtual = 1 << 9,
    HasDefault = 1 << 10,
    Magic = 1 << 11,
    Asymmetric = 1 << 12,
    Writeonly = 1 << 13,
}

/// A declared property of a class-like.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct PropertyMember<'arena> {
    /// The span of the property.
    pub span: Span,
    /// The visibility governing reads and writes to the property.
    pub visibility: ReadWriteVisibility,
    /// The name of the property (`Class::$x`).
    pub name: Path<'arena>,
    /// The identifier of the symbol that defines this property.
    pub defining_symbol: SymbolId,
    /// The flags of the property.
    pub flags: U16Flags<PropertyFlag>,
    /// The constraint of the property.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the property.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The type of the property: native hint, `@var` annotation, and the
    /// inferred type of its default, layered in [`TypeSlot`].
    pub ty: TypeSlot<'arena>,
    /// The property's `get` and `set` hooks, if any.
    pub hooks: &'arena [PropertyHookMember<'arena>],
    /// The origin of the property.
    pub origin: Origin,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct PropertyOverride<'arena> {
    /// Offset into `members` of the overriding property.
    pub member: u32,
    /// The ancestor property ids it overrides (a slice: diamonds let one property override several).
    pub overrides: &'arena [SymbolId],
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct PropertyMemberList<'arena> {
    /// Flattened: own + inherited, source order.
    pub members: &'arena [PropertyMember<'arena>],
    /// SymbolId-sorted offsets into `members`; O(log n) lookup by id.
    pub index: &'arena [u32],
    /// For each member that overrides ancestors: its offset + the ids it overrides.
    pub overrides: &'arena [PropertyOverride<'arena>],
}

impl<'arena> PropertyMemberList<'arena> {
    /// The flattened members in source order.
    #[inline]
    #[must_use]
    pub const fn members(&self) -> &'arena [PropertyMember<'arena>] {
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
    pub fn get(&self, id: SymbolId) -> Option<&'arena PropertyMember<'arena>> {
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

    /// The ancestor properties that the member with the given identifier overrides.
    #[must_use]
    pub fn overrides_of(&self, id: SymbolId) -> &'arena [SymbolId] {
        let members = self.members;
        for entry in self.overrides {
            if members.get(entry.member as usize).is_some_and(|member| member.name.id == id) {
                return entry.overrides;
            }
        }

        &[]
    }

    /// An iterator over the members in source order.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'arena, PropertyMember<'arena>> {
        self.members.iter()
    }
}

impl<'arena> IntoIterator for &PropertyMemberList<'arena> {
    type Item = &'arena PropertyMember<'arena>;
    type IntoIter = std::slice::Iter<'arena, PropertyMember<'arena>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'arena> SymbolMember<'arena> for PropertyMember<'arena> {
    fn path(&self) -> Path<'arena> {
        self.name
    }

    fn defining_symbol(&self) -> SymbolId {
        self.defining_symbol
    }

    fn origin(&self) -> Origin {
        self.origin
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        Some(self.constraint)
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        self.attributes
    }
}

impl PropertyMember<'_> {
    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::Deprecated as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::Internal as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_api(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::API as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::Experimental as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::Static as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::Readonly as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_final(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::Final as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_abstract(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::Abstract as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_promoted(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::Promoted as u16)
    }

    #[inline]
    #[must_use]
    pub const fn is_virtual(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::Virtual as u16)
    }

    #[inline]
    #[must_use]
    pub const fn has_default(&self) -> bool {
        self.flags.contains_bits(PropertyFlag::HasDefault as u16)
    }
}

impl From<PropertyFlag> for u16 {
    #[inline]
    fn from(flag: PropertyFlag) -> Self {
        flag as u16
    }
}

impl HasSpan for PropertyMember<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
