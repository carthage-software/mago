use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Unary<'arena> {
    pub operator: UnaryOperator<'arena>,
    pub operand: &'arena Expression<'arena>,
}

impl HasSpan for Unary<'_> {
    fn span(&self) -> Span {
        self.operator.span().join(self.operand.span())
    }
}
