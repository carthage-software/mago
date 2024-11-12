use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_span::HasSpan;
use fennec_span::Span;
use fennec_token::GetPrecedence;
use fennec_token::Precedence;

use crate::ast::expression::Expression;

/// Represents a PHP bitwise operation.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum BitwiseOperation {
    Prefix(BitwisePrefixOperation),
    Infix(BitwiseInfixOperation),
}

/// Represents a PHP bitwise prefix operator.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum BitwisePrefixOperator {
    Not(Span),
}

/// Represents a PHP bitwise prefix operation.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct BitwisePrefixOperation {
    pub operator: BitwisePrefixOperator,
    pub value: Expression,
}

/// Represents a PHP bitwise infix operator.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum BitwiseInfixOperator {
    And(Span),
    Or(Span),
    Xor(Span),
    LeftShift(Span),
    RightShift(Span),
}

/// Represents a PHP bitwise infix operation.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct BitwiseInfixOperation {
    pub lhs: Expression,
    pub operator: BitwiseInfixOperator,
    pub rhs: Expression,
}

impl BitwiseInfixOperator {
    #[inline]
    pub fn is_and(&self) -> bool {
        matches!(self, BitwiseInfixOperator::And(_))
    }

    #[inline]
    pub fn is_or(&self) -> bool {
        matches!(self, BitwiseInfixOperator::Or(_))
    }

    #[inline]
    pub fn is_xor(&self) -> bool {
        matches!(self, BitwiseInfixOperator::Xor(_))
    }

    #[inline]
    pub fn is_shift(&self) -> bool {
        matches!(self, BitwiseInfixOperator::LeftShift(_) | BitwiseInfixOperator::RightShift(_))
    }

    #[inline]
    pub fn is_left_shift(&self) -> bool {
        matches!(self, BitwiseInfixOperator::LeftShift(_))
    }

    #[inline]
    pub fn is_right_shift(&self) -> bool {
        matches!(self, BitwiseInfixOperator::RightShift(_))
    }

    #[inline]
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::And(_), Self::And(_))
            | (Self::Or(_), Self::Or(_))
            | (Self::Xor(_), Self::Xor(_))
            | (Self::LeftShift(_), Self::LeftShift(_))
            | (Self::RightShift(_), Self::RightShift(_)) => true,
            _ => false,
        }
    }
}

impl GetPrecedence for BitwiseInfixOperator {
    #[inline]
    fn precedence(&self) -> Precedence {
        match self {
            BitwiseInfixOperator::And(_) => Precedence::BitwiseAnd,
            BitwiseInfixOperator::Or(_) => Precedence::BitwiseOr,
            BitwiseInfixOperator::Xor(_) => Precedence::BitwiseXor,
            BitwiseInfixOperator::LeftShift(_) | BitwiseInfixOperator::RightShift(_) => Precedence::BitShift,
        }
    }
}

impl HasSpan for BitwiseOperation {
    fn span(&self) -> Span {
        match self {
            BitwiseOperation::Prefix(operation) => operation.span(),
            BitwiseOperation::Infix(operation) => operation.span(),
        }
    }
}

impl HasSpan for BitwisePrefixOperator {
    fn span(&self) -> Span {
        match self {
            BitwisePrefixOperator::Not(span) => *span,
        }
    }
}

impl HasSpan for BitwisePrefixOperation {
    fn span(&self) -> Span {
        self.operator.span().join(self.value.span())
    }
}

impl HasSpan for BitwiseInfixOperator {
    fn span(&self) -> Span {
        match self {
            BitwiseInfixOperator::And(span) => *span,
            BitwiseInfixOperator::Or(span) => *span,
            BitwiseInfixOperator::Xor(span) => *span,
            BitwiseInfixOperator::LeftShift(span) => *span,
            BitwiseInfixOperator::RightShift(span) => *span,
        }
    }
}

impl HasSpan for BitwiseInfixOperation {
    fn span(&self) -> Span {
        self.lhs.span().join(self.rhs.span())
    }
}
