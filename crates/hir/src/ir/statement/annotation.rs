#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct VariableBindingAnnotation<'arena> {
    pub span: Span,
    pub variable: DirectVariable<'arena>,
    pub type_annotation: &'arena TypeAnnotation<'arena>,
}

impl HasSpan for VariableBindingAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for VariableBindingAnnotation<'_> {
    type Output<'arena> = VariableBindingAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        VariableBindingAnnotation {
            span: self.span,
            variable: self.variable.copy_into(arena),
            type_annotation: copy_ref_into(self.type_annotation, arena),
        }
    }
}
