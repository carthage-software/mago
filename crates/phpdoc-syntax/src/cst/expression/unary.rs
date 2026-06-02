use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::expression::ConstantExpression;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum UnaryPrefixConstantOperator {
    Plus(Span),
    Negation(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UnaryPrefixConstantExpression<'arena> {
    pub operator: UnaryPrefixConstantOperator,
    pub operand: &'arena ConstantExpression<'arena>,
}

impl HasSpan for UnaryPrefixConstantOperator {
    fn span(&self) -> Span {
        match self {
            Self::Plus(span) | Self::Negation(span) => *span,
        }
    }
}

impl HasSpan for UnaryPrefixConstantExpression<'_> {
    fn span(&self) -> Span {
        self.operator.span().join(self.operand.span())
    }
}
