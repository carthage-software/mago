#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;

use crate::ir::expression::Expression;
use crate::ir::variable::annotation::VariableAnnotation;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Annotation<'arena, I, S, E> {
    pub annotation: &'arena VariableAnnotation<'arena>,
    pub expression: &'arena Expression<'arena, I, S, E>,
}

impl<I, S, E> CopyInto for Annotation<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Annotation<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Annotation {
            annotation: copy_ref_into(self.annotation, arena),
            expression: copy_ref_into(self.expression, arena),
        }
    }
}
