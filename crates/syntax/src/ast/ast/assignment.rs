use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;

/// Represents a PHP assignment operator.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Assignment<'arena> {
    pub lhs: &'arena Expression<'arena>,
    pub operator: AssignmentOperator,
    pub rhs: &'arena Expression<'arena>,
}

impl AssignmentOperator {
    #[inline]
    pub const fn is_assign(&self) -> bool {
        matches!(self, Self::Assign(_))
    }

    #[inline]
    pub const fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Self::Addition(_) | Self::Subtraction(_) | Self::Multiplication(_) | Self::Division(_) | Self::Modulo(_)
        )
    }

    #[inline]
    pub const fn is_bitwise(&self) -> bool {
        matches!(
            self,
            Self::BitwiseAnd(_) | Self::BitwiseOr(_) | Self::BitwiseXor(_) | Self::LeftShift(_) | Self::RightShift(_)
        )
    }

    /// Returns the string representation of the assignment operator.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Assign(_) => "=",
            Self::Addition(_) => "+=",
            Self::Subtraction(_) => "-=",
            Self::Multiplication(_) => "*=",
            Self::Division(_) => "/=",
            Self::Modulo(_) => "%=",
            Self::Exponentiation(_) => "**=",
            Self::Concat(_) => ".=",
            Self::BitwiseAnd(_) => "&=",
            Self::BitwiseOr(_) => "|=",
            Self::BitwiseXor(_) => "^=",
            Self::LeftShift(_) => "<<=",
            Self::RightShift(_) => ">>=",
            Self::Coalesce(_) => "??=",
        }
    }
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
