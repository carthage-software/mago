use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::cst::statement::Statement;
use crate::cst::sequence::Sequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
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
