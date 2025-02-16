use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;

/// Represents a PHP assignment operator.
#[derive(Debug, Hash, Serialize, Display)]
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Assignment<'a> {
    pub lhs: Box<'a, Expression<'a>>,
    pub operator: AssignmentOperator,
    pub rhs: Box<'a, Expression<'a>>,
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

impl HasSpan for Assignment<'_> {
    fn span(&self) -> Span {
        self.lhs.span().join(self.rhs.span())
    }
}
