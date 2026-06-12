#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::Span;

use crate::ir::item::expression::anonymous_class::AnonymousClass;
use crate::ir::item::expression::arrow_function::ArrowFunction;
use crate::ir::item::expression::closure::Closure;

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
