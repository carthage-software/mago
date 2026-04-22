use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ParenthesizedType<'arena> {
    pub left_parenthesis: Span,
    pub inner: &'arena Type<'arena>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UnionType<'arena> {
    pub left: &'arena Type<'arena>,
    pub pipe: Span,
    pub right: &'arena Type<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IntersectionType<'arena> {
    pub left: &'arena Type<'arena>,
    pub ampersand: Span,
    pub right: &'arena Type<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NullableType<'arena> {
    pub question_mark: Span,
    pub inner: &'arena Type<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TrailingPipeType<'arena> {
    pub inner: &'arena Type<'arena>,
    pub pipe: Span,
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

impl HasSpan for TrailingPipeType<'_> {
    fn span(&self) -> Span {
        self.inner.span().join(self.pipe)
    }
}

impl std::fmt::Display for ParenthesizedType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.inner)
    }
}

impl std::fmt::Display for UnionType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {}", self.left, self.right)
    }
}

impl std::fmt::Display for IntersectionType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} & {}", self.left, self.right)
    }
}

impl std::fmt::Display for NullableType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?{}", self.inner)
    }
}

impl std::fmt::Display for TrailingPipeType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|", self.inner)
    }
}
