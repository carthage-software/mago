use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Print<'arena> {
    pub open_variable: Span,
    pub expression: Expression<'arena>,
    pub close_variable: Span,
}

impl HasSpan for Print<'_> {
    fn span(&self) -> Span {
        self.open_variable.join(self.close_variable)
    }
}
