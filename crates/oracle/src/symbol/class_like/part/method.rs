use std::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U32Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::SymbolMember;
use crate::symbol::class_like::part::visibility::Visibility;
use crate::symbol::function_like::part::assertion::FunctionLikeAssertion;
use crate::symbol::function_like::part::parameter::SignatureParameter;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::generic::GenericParameter;
use crate::symbol::part::generic::WhereConstraint;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;
use crate::ty::Type;
use crate::var::Var;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u32)]
pub enum MethodFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    HasYield = 1 << 4,
    HasThrow = 1 << 5,
    MustUse = 1 << 6,
    Pure = 1 << 7,
    IgnoreNullableReturn = 1 << 8,
    IgnoreFalsableReturn = 1 << 9,
    NoNamedArguments = 1 << 10,
    ReturnsByReference = 1 << 11,
    SuspendsFiber = 1 << 12,
    AssertionsInferred = 1 << 13,
    Static = 1 << 14,
    Final = 1 << 15,
    Abstract = 1 << 16,
    Constructor = 1 << 17,
    Magic = 1 << 18,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct MethodMember<'arena> {
    /// The span of the method.
    pub span: Span,
    /// The visibility of the method.
    pub visibility: Visibility,
    /// The name of the method.
    pub name: Path<'arena>,
    /// The identifier of the symbol that defines this method.
    pub defining_symbol: SymbolId,
    /// The flags of the method.
    pub flags: U32Flags<MethodFlag>,
    /// The constraint of the method.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the method.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The generic parameters of the method.
    pub generics: &'arena [GenericParameter<'arena>],
    /// The parameters of the method.
    pub params: &'arena [SignatureParameter<'arena>],
    /// The return type of the method.
    pub ret: TypeSlot<'arena>,
    /// The where constraints of the method.
    pub where_constraints: &'arena [WhereConstraint<'arena>],
    /// The types that the method can throw.
    pub throws: &'arena [Type<'arena>],
    /// The assertions that the method makes about its parameters.
    pub assertions: &'arena [FunctionLikeAssertion<'arena>],
    /// The global variables that the method symbol accesses.
    pub accessed_globals: &'arena [Var<'arena>],
    /// The origin of the symbol.
    pub origin: Origin,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct MethodOverride<'arena> {
    /// Offset into `members` of the overriding method.
    pub member: u32,
    /// The ancestor method ids it overrides (a slice: diamonds let one method override several).
    pub overrides: &'arena [SymbolId],
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct MethodMemberList<'arena> {
    /// Flattened: own + inherited + materialized `@method`, source order.
    pub members: &'arena [MethodMember<'arena>],
    /// SymbolId-sorted offsets into `members`; O(log n) lookup by id.
    pub index: &'arena [u32],
    /// For each member that overrides ancestors: its offset + the ids it overrides.
    pub overrides: &'arena [MethodOverride<'arena>],
}

impl<'arena> MethodMemberList<'arena> {
    /// The flattened members in source order.
    #[inline]
    #[must_use]
    pub const fn members(&self) -> &'arena [MethodMember<'arena>] {
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
    pub fn get(&self, id: SymbolId) -> Option<&'arena MethodMember<'arena>> {
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

    /// The ancestor methods that the member with the given identifier overrides.
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
    pub fn iter(&self) -> std::slice::Iter<'arena, MethodMember<'arena>> {
        self.members.iter()
    }
}

impl<'arena> IntoIterator for &MethodMemberList<'arena> {
    type Item = &'arena MethodMember<'arena>;
    type IntoIter = std::slice::Iter<'arena, MethodMember<'arena>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'arena> SymbolMember<'arena> for MethodMember<'arena> {
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

impl MethodMember<'_> {
    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.flags.contains_bits(MethodFlag::Deprecated as u32)
    }

    #[inline]
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        self.flags.contains_bits(MethodFlag::Internal as u32)
    }

    #[inline]
    #[must_use]
    pub const fn is_api(&self) -> bool {
        self.flags.contains_bits(MethodFlag::API as u32)
    }

    #[inline]
    #[must_use]
    pub const fn is_experimental(&self) -> bool {
        self.flags.contains_bits(MethodFlag::Experimental as u32)
    }
}

impl From<MethodFlag> for u32 {
    #[inline]
    fn from(flag: MethodFlag) -> Self {
        flag as u32
    }
}

impl HasSpan for MethodMember<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
