use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::ir::item::statement::class::Class;
use crate::ir::item::statement::constant::Constant;
use crate::ir::item::statement::r#enum::Enum;
use crate::ir::item::statement::function::Function;
use crate::ir::item::statement::interface::Interface;
use crate::ir::item::statement::r#trait::Trait;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

pub mod class;
pub mod constant;
pub mod r#enum;
pub mod function;
pub mod interface;
pub mod r#trait;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ItemStatement<'arena, I, S, E> {
    pub meta: I,
    pub span: Span,
    pub kind: ItemStatementKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum ItemStatementKind<'arena, I, S, E> {
    Class(&'arena Class<'arena, I, S, E>),
    Interface(&'arena Interface<'arena, I, S, E>),
    Trait(&'arena Trait<'arena, I, S, E>),
    Enum(&'arena Enum<'arena, I, S, E>),
    Constant(&'arena Constant<'arena, I, S, E>),
    Function(&'arena Function<'arena, I, S, E>),
}

impl<I, S, E> CopyInto for ItemStatement<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ItemStatement<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ItemStatement { meta: self.meta.copy_into(arena), span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for ItemStatementKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ItemStatementKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            ItemStatementKind::Class(node) => ItemStatementKind::Class(copy_ref_into(*node, arena)),
            ItemStatementKind::Interface(node) => ItemStatementKind::Interface(copy_ref_into(*node, arena)),
            ItemStatementKind::Trait(node) => ItemStatementKind::Trait(copy_ref_into(*node, arena)),
            ItemStatementKind::Enum(node) => ItemStatementKind::Enum(copy_ref_into(*node, arena)),
            ItemStatementKind::Constant(node) => ItemStatementKind::Constant(copy_ref_into(*node, arena)),
            ItemStatementKind::Function(node) => ItemStatementKind::Function(copy_ref_into(*node, arena)),
        }
    }
}

impl<I, S, E> HasSpan for ItemStatement<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
