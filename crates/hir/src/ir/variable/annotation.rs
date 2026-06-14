#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::error::annotation::AnnotationError;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct VariableAnnotation<'arena> {
    pub span: Span,
    pub type_annotation: &'arena TypeAnnotation<'arena>,
    pub variable: Option<DirectVariable<'arena>>,
    pub errors: &'arena [AnnotationError],
}

impl CopyInto for VariableAnnotation<'_> {
    type Output<'arena> = VariableAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        VariableAnnotation {
            span: self.span,
            type_annotation: copy_ref_into(self.type_annotation, arena),
            variable: self.variable.map(|variable| variable.copy_into(arena)),
            errors: arena.alloc_slice_copy(self.errors),
        }
    }
}

impl HasSpan for VariableAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
