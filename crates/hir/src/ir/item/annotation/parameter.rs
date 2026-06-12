#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ParameterAnnotation<'arena, I, S, E> {
    pub span: Span,
    pub r#type: Option<&'arena TypeAnnotation<'arena>>,
    pub is_by_reference: bool,
    pub is_variadic: bool,
    pub variable: DirectVariable<'arena>,
    pub default_value: Option<&'arena Expression<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ParameterOutAnnotation<'arena> {
    pub span: Span,
    pub r#type: &'arena TypeAnnotation<'arena>,
    pub variable: DirectVariable<'arena>,
}

impl<I, S, E> HasSpan for ParameterAnnotation<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ParameterOutAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for ParameterOutAnnotation<'_> {
    type Output<'arena> = ParameterOutAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ParameterOutAnnotation {
            span: self.span,
            r#type: copy_ref_into(self.r#type, arena),
            variable: self.variable.copy_into(arena),
        }
    }
}
