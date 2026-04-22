use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;

/// A Twig conditional expression.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Conditional<'arena> {
    pub condition: &'arena Expression<'arena>,
    pub question_mark: Span,
    pub then: Option<&'arena Expression<'arena>>,
    pub colon: Option<Span>,
    pub r#else: Option<&'arena Expression<'arena>>,
}

impl HasSpan for Conditional<'_> {
    fn span(&self) -> Span {
        let end = self
            .r#else
            .map(HasSpan::span)
            .or(self.colon)
            .or_else(|| self.then.map(HasSpan::span))
            .unwrap_or(self.question_mark);
        self.condition.span().join(end)
    }
}
