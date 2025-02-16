use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::statement::Statement;
use crate::sequence::Sequence;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Block<'a> {
    pub left_brace: Span,
    pub statements: Sequence<'a, Statement<'a>>,
    pub right_brace: Span,
}

impl HasSpan for Block<'_> {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}
