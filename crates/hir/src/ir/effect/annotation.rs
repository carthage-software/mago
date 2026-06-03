use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::name::Name;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ThrowsAnnotation<'arena> {
    pub span: Span,
    pub types: &'arena [TypeAnnotation<'arena>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct AssertAnnotation<'arena> {
    pub span: Span,
    pub r#type: &'arena TypeAnnotation<'arena>,
    pub target: AssertAnnotationTarget<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum AssertAnnotationTarget<'arena> {
    Variable(DirectVariable<'arena>),
    Method(DirectVariable<'arena>, Name<'arena>),
    Property(DirectVariable<'arena>, Name<'arena>),
}

impl HasSpan for ThrowsAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for AssertAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
