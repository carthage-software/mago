use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Do<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub expression: Expression<'arena>,
    pub close_tag: Span,
}

impl HasSpan for Do<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.close_tag)
    }
}
