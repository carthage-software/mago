use serde::Deserialize;
use serde::Serialize;

use fennec_span::HasSpan;
use fennec_span::Span;

use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Conditional {
    pub condition: Box<Expression>,
    pub question_mark: Span,
    pub then: Option<Box<Expression>>,
    pub colon: Span,
    pub r#else: Box<Expression>,
}

impl HasSpan for Conditional {
    fn span(&self) -> Span {
        self.condition.span().join(self.r#else.span())
    }
}
