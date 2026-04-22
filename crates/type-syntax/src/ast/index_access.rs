use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IndexAccessType<'arena> {
    pub target: &'arena Type<'arena>,
    pub left_bracket: Span,
    pub index: &'arena Type<'arena>,
    pub right_bracket: Span,
}

impl HasSpan for IndexAccessType<'_> {
    fn span(&self) -> Span {
        self.target.span().join(self.right_bracket)
    }
}

impl std::fmt::Display for IndexAccessType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.target, self.index)
    }
}
