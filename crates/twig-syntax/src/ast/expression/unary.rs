use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum UnaryOperator<'arena> {
    MinusSign(Span),
    PlusSign(Span),
    Not(Keyword<'arena>),
}

impl HasSpan for UnaryOperator<'_> {
    fn span(&self) -> Span {
        match self {
            UnaryOperator::MinusSign(s) | UnaryOperator::PlusSign(s) => *s,
            UnaryOperator::Not(k) => k.span,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Unary<'arena> {
    pub operator: UnaryOperator<'arena>,
    pub operand: &'arena Expression<'arena>,
}

impl HasSpan for Unary<'_> {
    fn span(&self) -> Span {
        self.operator.span().join(self.operand.span())
    }
}
