use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::statement::Statement;
use crate::sequence::Sequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Block {
    pub left_brace: Span,
    pub statements: Sequence<Statement>,
    pub right_brace: Span,
}

impl HasSpan for Block {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}
