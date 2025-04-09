use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Unset {
    pub unset: Keyword,
    pub left_parenthesis: Span,
    pub values: TokenSeparatedSequence<Expression>,
    pub right_parenthesis: Span,
    pub terminator: Terminator,
}

impl HasSpan for Unset {
    fn span(&self) -> Span {
        self.unset.span().join(self.terminator.span())
    }
}
