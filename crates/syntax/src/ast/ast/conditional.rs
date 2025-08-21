use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Conditional<'arena> {
    pub condition: &'arena Expression<'arena>,
    pub question_mark: Span,
    pub then: Option<&'arena Expression<'arena>>,
    pub colon: Span,
    pub r#else: &'arena Expression<'arena>,
}

impl HasSpan for Conditional<'_> {
    fn span(&self) -> Span {
        self.condition.span().join(self.r#else.span())
    }
}
