#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::Span;

use crate::ir::item::expression::anonymous_class::AnonymousClass;
use crate::ir::item::expression::arrow_function::ArrowFunction;
use crate::ir::item::expression::closure::Closure;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

pub mod anonymous_class;
pub mod arrow_function;
pub mod closure;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ItemExpression<'arena, I, S, E> {
    pub meta: I,
    pub span: Span,
    pub kind: ItemExpressionKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum ItemExpressionKind<'arena, I, S, E> {
    AnonymousClass(&'arena AnonymousClass<'arena, I, S, E>),
    ArrowFunction(&'arena ArrowFunction<'arena, I, S, E>),
    Closure(&'arena Closure<'arena, I, S, E>),
}

impl<I, S, E> CopyInto for ItemExpression<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ItemExpression<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ItemExpression { meta: self.meta.copy_into(arena), span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for ItemExpressionKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ItemExpressionKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            ItemExpressionKind::AnonymousClass(node) => ItemExpressionKind::AnonymousClass(copy_ref_into(*node, arena)),
            ItemExpressionKind::ArrowFunction(node) => ItemExpressionKind::ArrowFunction(copy_ref_into(*node, arena)),
            ItemExpressionKind::Closure(node) => ItemExpressionKind::Closure(copy_ref_into(*node, arena)),
        }
    }
}
