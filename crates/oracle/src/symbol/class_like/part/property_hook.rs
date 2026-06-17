#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::SymbolMember;
use crate::symbol::function_like::part::parameter::SignatureParameter;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum HookKind {
    Get,
    Set,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum PropertyHookFlag {
    Deprecated = 1 << 0,
    Internal = 1 << 1,
    API = 1 << 2,
    Experimental = 1 << 3,
    Final = 1 << 4,
    Abstract = 1 << 5,
    ReturnsByReference = 1 << 6,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct PropertyHookMember<'arena> {
    /// The span of the hook.
    pub span: Span,
    /// Whether this is the `get` or `set` hook.
    pub kind: HookKind,
    /// The name of the hook (`Class::$prop::get`).
    pub name: Path<'arena>,
    /// The identifier of the property that defines this hook.
    pub defining_symbol: SymbolId,
    /// The flags of the hook.
    pub flags: U8Flags<PropertyHookFlag>,
    /// The constraint of the hook.
    pub constraint: SymbolConstraint<'arena>,
    /// The attributes of the hook.
    pub attributes: &'arena [AppliedAttribute<'arena>],
    /// The value parameter of a `set` hook (explicit, or the implicit `$value`).
    pub parameter: Option<SignatureParameter<'arena>>,
    /// The value the hook reads or writes: a `get` hook's return type, or a `set` hook's accepted type.
    pub ty: TypeSlot<'arena>,
    /// The origin of the hook.
    pub origin: Origin,
}

impl<'arena> SymbolMember<'arena> for PropertyHookMember<'arena> {
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

impl PropertyHookMember<'_> {
    #[inline]
    #[must_use]
    pub const fn is_get(&self) -> bool {
        matches!(self.kind, HookKind::Get)
    }

    #[inline]
    #[must_use]
    pub const fn is_set(&self) -> bool {
        matches!(self.kind, HookKind::Set)
    }

    #[inline]
    #[must_use]
    pub const fn is_final(&self) -> bool {
        self.flags.contains_bits(PropertyHookFlag::Final as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_abstract(&self) -> bool {
        self.flags.contains_bits(PropertyHookFlag::Abstract as u8)
    }

    #[inline]
    #[must_use]
    pub const fn returns_by_reference(&self) -> bool {
        self.flags.contains_bits(PropertyHookFlag::ReturnsByReference as u8)
    }
}

impl From<PropertyHookFlag> for u8 {
    #[inline]
    fn from(flag: PropertyHookFlag) -> Self {
        flag as u8
    }
}

impl HasSpan for PropertyHookMember<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
