use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Throw<'arena> {
    pub throw: Keyword<'arena>,
    pub exception: &'arena Expression<'arena>,
}

impl HasSpan for Throw<'_> {
    fn span(&self) -> Span {
        self.throw.span().join(self.exception.span())
    }
}
