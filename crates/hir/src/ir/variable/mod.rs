#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;
use mago_allocator::copy::CopyInto;

pub mod annotation;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Variable<'arena, I, S, E> {
    Direct(DirectVariable<'arena>),
    Indirect(&'arena Expression<'arena, I, S, E>),
    Nested(&'arena Variable<'arena, I, S, E>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct DirectVariable<'arena> {
    pub span: Span,
    pub name: &'arena [u8],
}

impl HasSpan for DirectVariable<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Variable<'_, I, S, E> {
    fn span(&self) -> Span {
        match self {
            Variable::Direct(variable) => variable.span(),
            Variable::Indirect(expression) => expression.span(),
            Variable::Nested(variable) => variable.span(),
        }
    }
}

impl CopyInto for DirectVariable<'_> {
    type Output<'arena> = DirectVariable<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        DirectVariable { span: self.span, name: arena.alloc_slice_copy(self.name) }
    }
}
