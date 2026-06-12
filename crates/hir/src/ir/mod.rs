#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::error::Error;
use crate::ir::statement::Statement;

pub mod argument;
pub mod delimited;
pub mod error;
pub mod expression;
pub mod identifier;
pub mod item;
pub mod literal;
pub mod name;
pub mod statement;
pub mod r#type;
pub mod variable;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct IR<'arena, I, S, E> {
    pub span: Span,
    pub statements: &'arena [Statement<'arena, I, S, E>],
    pub errors: &'arena [Error],
}

impl<I, S, E> HasSpan for IR<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
