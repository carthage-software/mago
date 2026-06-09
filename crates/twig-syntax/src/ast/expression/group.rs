use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Parenthesized<'arena> {
    pub left_parenthesis: Span,
    pub inner: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
}

impl HasSpan for Parenthesized<'_> {
    fn span(&self) -> Span {
        self.left_parenthesis.join(self.right_parenthesis)
    }
}
