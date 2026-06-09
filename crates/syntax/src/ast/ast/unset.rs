use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
