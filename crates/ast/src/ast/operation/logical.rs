use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_span::HasSpan;
use fennec_span::Span;
use fennec_token::GetPrecedence;
use fennec_token::Precedence;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum LogicalOperation {
    Prefix(LogicalPrefixOperation),
    Infix(LogicalInfixOperation),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum LogicalPrefixOperator {
    Not(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct LogicalPrefixOperation {
    pub operator: LogicalPrefixOperator,
    pub value: Expression,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum LogicalInfixOperator {
    And(Span),
    Or(Span),
    LowPrecedenceAnd(Keyword),
    LowPrecedenceOr(Keyword),
    LowPrecedenceXor(Keyword),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct LogicalInfixOperation {
    pub lhs: Expression,
    pub operator: LogicalInfixOperator,
    pub rhs: Expression,
}

impl LogicalInfixOperator {
    #[inline]
    pub fn is_and(&self) -> bool {
        matches!(self, LogicalInfixOperator::And(_))
    }

    #[inline]
    pub fn is_or(&self) -> bool {
        matches!(self, LogicalInfixOperator::Or(_))
    }

    #[inline]
    pub fn is_low_precedence_and(&self) -> bool {
        matches!(self, LogicalInfixOperator::LowPrecedenceAnd(_))
    }

    #[inline]
    pub fn is_low_precedence_or(&self) -> bool {
        matches!(self, LogicalInfixOperator::LowPrecedenceOr(_))
    }

    #[inline]
    pub fn is_low_precedence_xor(&self) -> bool {
        matches!(self, LogicalInfixOperator::LowPrecedenceXor(_))
    }

    #[inline]
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::And(_), Self::And(_))
            | (Self::Or(_), Self::Or(_))
            | (Self::LowPrecedenceAnd(_), Self::LowPrecedenceAnd(_))
            | (Self::LowPrecedenceOr(_), Self::LowPrecedenceOr(_))
            | (Self::LowPrecedenceXor(_), Self::LowPrecedenceXor(_)) => true,
            _ => false,
        }
    }
}

impl GetPrecedence for LogicalInfixOperator {
    fn precedence(&self) -> Precedence {
        match self {
            LogicalInfixOperator::And(_) => Precedence::And,
            LogicalInfixOperator::Or(_) => Precedence::Or,
            LogicalInfixOperator::LowPrecedenceAnd(_) => Precedence::KeyAnd,
            LogicalInfixOperator::LowPrecedenceOr(_) => Precedence::KeyOr,
            LogicalInfixOperator::LowPrecedenceXor(_) => Precedence::KeyXor,
        }
    }
}

impl HasSpan for LogicalOperation {
    fn span(&self) -> Span {
        match self {
            LogicalOperation::Prefix(operation) => operation.span(),
            LogicalOperation::Infix(operation) => operation.span(),
        }
    }
}

impl HasSpan for LogicalPrefixOperator {
    fn span(&self) -> Span {
        match self {
            LogicalPrefixOperator::Not(span) => *span,
        }
    }
}

impl HasSpan for LogicalPrefixOperation {
    fn span(&self) -> Span {
        self.operator.span().join(self.value.span())
    }
}

impl HasSpan for LogicalInfixOperator {
    fn span(&self) -> Span {
        match self {
            LogicalInfixOperator::And(span) => *span,
            LogicalInfixOperator::Or(span) => *span,
            LogicalInfixOperator::LowPrecedenceAnd(keyword) => keyword.span(),
            LogicalInfixOperator::LowPrecedenceOr(keyword) => keyword.span(),
            LogicalInfixOperator::LowPrecedenceXor(keyword) => keyword.span(),
        }
    }
}

impl HasSpan for LogicalInfixOperation {
    fn span(&self) -> Span {
        self.lhs.span().join(self.rhs.span())
    }
}
