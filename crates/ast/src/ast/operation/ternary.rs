use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_span::HasSpan;
use fennec_span::Span;

use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum TernaryOperation {
    Conditional(ConditionalTernaryOperation),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ConditionalTernaryOperation {
    pub condition: Expression,
    pub question_mark: Span,
    pub then: Option<Expression>,
    pub colon: Span,
    pub r#else: Expression,
}

impl HasSpan for TernaryOperation {
    fn span(&self) -> Span {
        match self {
            TernaryOperation::Conditional(conditional) => conditional.span(),
        }
    }
}

impl HasSpan for ConditionalTernaryOperation {
    fn span(&self) -> Span {
        self.condition.span().join(self.r#else.span())
    }
}
