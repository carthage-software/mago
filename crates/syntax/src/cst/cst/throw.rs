use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::cst::expression::Expression;
use crate::cst::cst::keyword::Keyword;

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
