use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::expression::ConstantExpression;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum UnaryPrefixConstantOperator {
    Plus(Span),
    Negation(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
