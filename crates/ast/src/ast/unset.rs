use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Unset<'a> {
    pub unset: Keyword,
    pub left_parenthesis: Span,
    pub values: TokenSeparatedSequence<'a, Expression<'a>>,
    pub right_parenthesis: Span,
    pub terminator: Terminator,
}

impl HasSpan for Unset<'_> {
    fn span(&self) -> Span {
        self.unset.span().join(self.terminator.span())
    }
}
