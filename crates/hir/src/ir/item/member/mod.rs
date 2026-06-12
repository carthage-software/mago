use mago_span::HasSpan;
#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::Span;

use crate::ir::item::member::constant::ClassLikeConstant;
use crate::ir::item::member::enum_case::EnumCase;
use crate::ir::item::member::method::Method;
use crate::ir::item::member::property::HookedProperty;
use crate::ir::item::member::property::Property;
use crate::ir::item::member::trait_use::TraitUse;

pub mod constant;
pub mod enum_case;
pub mod hook;
pub mod method;
pub mod property;
pub mod trait_use;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct MemberItem<'arena, I, S, E> {
    pub meta: I,
    pub span: Span,
    pub kind: MemberItemKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum MemberItemKind<'arena, I, S, E> {
    Method(&'arena Method<'arena, I, S, E>),
    Property(&'arena Property<'arena, I, S, E>),
    HookedProperty(&'arena HookedProperty<'arena, I, S, E>),
    TraitUse(&'arena TraitUse<'arena, I, S, E>),
    Constant(&'arena ClassLikeConstant<'arena, I, S, E>),
    EnumCase(&'arena EnumCase<'arena, I, S, E>),
}

impl<I, S, E> HasSpan for MemberItem<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
