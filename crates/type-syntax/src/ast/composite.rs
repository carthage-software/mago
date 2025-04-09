use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ParenthesizedType<'input> {
    pub left_parenthesis: Span,
    pub inner: Box<Type<'input>>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct UnionType<'input> {
    pub left: Box<Type<'input>>,
    pub pipe: Span,
    pub right: Box<Type<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct IntersectionType<'input> {
    pub left: Box<Type<'input>>,
    pub ampersand: Span,
    pub right: Box<Type<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct NullableType<'input> {
    pub question_mark: Span,
    pub inner: Box<Type<'input>>,
}

impl HasSpan for ParenthesizedType<'_> {
    fn span(&self) -> Span {
        self.left_parenthesis.join(self.right_parenthesis)
    }
}

impl HasSpan for UnionType<'_> {
    fn span(&self) -> Span {
        self.left.span().join(self.right.span())
    }
}

impl HasSpan for IntersectionType<'_> {
    fn span(&self) -> Span {
        self.left.span().join(self.right.span())
    }
}

impl HasSpan for NullableType<'_> {
    fn span(&self) -> Span {
        self.question_mark.join(self.inner.span())
    }
}
