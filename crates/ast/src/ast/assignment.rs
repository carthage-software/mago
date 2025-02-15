use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;

/// Represents a PHP assignment operator.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum AssignmentOperator {
    Assign(Span),
    Addition(Span),
    Subtraction(Span),
    Multiplication(Span),
    Division(Span),
    Modulo(Span),
    Exponentiation(Span),
    Concat(Span),
    BitwiseAnd(Span),
    BitwiseOr(Span),
    BitwiseXor(Span),
    LeftShift(Span),
    RightShift(Span),
    Coalesce(Span),
}

/// Represents a PHP assignment operation
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Assignment {
    pub lhs: Box<Expression>,
    pub operator: AssignmentOperator,
    pub rhs: Box<Expression>,
}

impl HasSpan for AssignmentOperator {
    fn span(&self) -> Span {
        match self {
            Self::Assign(span) => *span,
            Self::Addition(span) => *span,
            Self::Subtraction(span) => *span,
            Self::Multiplication(span) => *span,
            Self::Division(span) => *span,
            Self::Modulo(span) => *span,
            Self::Exponentiation(span) => *span,
            Self::Concat(span) => *span,
            Self::BitwiseAnd(span) => *span,
            Self::BitwiseOr(span) => *span,
            Self::BitwiseXor(span) => *span,
            Self::LeftShift(span) => *span,
            Self::RightShift(span) => *span,
            Self::Coalesce(span) => *span,
        }
    }
}

impl HasSpan for Assignment {
    fn span(&self) -> Span {
        self.lhs.span().join(self.rhs.span())
    }
}
