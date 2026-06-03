use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum Variable<'arena, S, D, E> {
    Direct(DirectVariable<'arena>),
    Indirect(&'arena Expression<'arena, S, D, E>),
    Nested(&'arena Variable<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DirectVariable<'arena> {
    pub span: Span,
    pub name: &'arena [u8],
}

impl HasSpan for DirectVariable<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
