use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct NegatedType<'input> {
    pub minus: Span,
    pub inner: Box<Type<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct PositedType<'input> {
    pub plus: Span,
    pub inner: Box<Type<'input>>,
}

impl HasSpan for NegatedType<'_> {
    fn span(&self) -> Span {
        self.minus.join(self.inner.span())
    }
}

impl HasSpan for PositedType<'_> {
    fn span(&self) -> Span {
        self.plus.join(self.inner.span())
    }
}
