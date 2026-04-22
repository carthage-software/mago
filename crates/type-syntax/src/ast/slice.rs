use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SliceType<'arena> {
    pub inner: &'arena Type<'arena>,
    pub left_bracket: Span,
    pub right_bracket: Span,
}

impl HasSpan for SliceType<'_> {
    fn span(&self) -> Span {
        self.inner.span().join(self.right_bracket)
    }
}

impl std::fmt::Display for SliceType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[]", self.inner)
    }
}
