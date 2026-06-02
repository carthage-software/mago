use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::cst::expression::Expression;
use crate::cst::cst::keyword::Keyword;
use crate::cst::cst::terminator::Terminator;
use crate::cst::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Unset<'arena> {
    pub unset: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub values: TokenSeparatedSequence<'arena, &'arena Expression<'arena>>,
    pub right_parenthesis: Span,
    pub terminator: Terminator<'arena>,
}

impl HasSpan for Unset<'_> {
    fn span(&self) -> Span {
        self.unset.span().join(self.terminator.span())
    }
}
