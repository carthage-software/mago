#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::error::Error;
use crate::ir::statement::Statement;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;

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

impl<I, S, E> CopyInto for IR<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = IR<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        IR {
            span: self.span,
            statements: copy_slice_into(self.statements, arena),
            errors: arena.alloc_slice_copy(self.errors),
        }
    }
}

impl<I, S, E> HasSpan for IR<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

const _: fn() = || {
    fn assert_copy_into<T>()
    where
        T: mago_allocator::copy::CopyInto,
    {
    }

    assert_copy_into::<IR<'static, (), (), ()>>();
};
