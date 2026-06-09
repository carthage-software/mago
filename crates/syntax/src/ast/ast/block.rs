use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::statement::Statement;
use crate::ast::sequence::Sequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Block<'arena> {
    pub left_brace: Span,
    pub statements: Sequence<'arena, Statement<'arena>>,
    pub right_brace: Span,
}

impl HasSpan for Block<'_> {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}
