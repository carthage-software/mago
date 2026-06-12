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

impl<I, S, E> HasSpan for ItemStatement<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
