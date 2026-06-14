use mago_allocator::Arena;
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
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

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

impl<I, S, E> CopyInto for MemberItem<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = MemberItem<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        MemberItem { meta: self.meta.copy_into(arena), span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for MemberItemKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = MemberItemKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            MemberItemKind::Method(node) => MemberItemKind::Method(copy_ref_into(*node, arena)),
            MemberItemKind::Property(node) => MemberItemKind::Property(copy_ref_into(*node, arena)),
            MemberItemKind::HookedProperty(node) => MemberItemKind::HookedProperty(copy_ref_into(*node, arena)),
            MemberItemKind::TraitUse(node) => MemberItemKind::TraitUse(copy_ref_into(*node, arena)),
            MemberItemKind::Constant(node) => MemberItemKind::Constant(copy_ref_into(*node, arena)),
            MemberItemKind::EnumCase(node) => MemberItemKind::EnumCase(copy_ref_into(*node, arena)),
        }
    }
}

impl<I, S, E> HasSpan for MemberItem<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
