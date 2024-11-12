use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_span::HasSpan;
use fennec_span::Span;
use fennec_token::GetPrecedence;
use fennec_token::Precedence;

use crate::ast::expression::Expression;

/// Represents a PHP comparison operator.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum ComparisonOperator {
    Equal(Span),
    NotEqual(Span),
    Identical(Span),
    NotIdentical(Span),
    AngledNotEqual(Span),
    LessThan(Span),
    LessThanOrEqual(Span),
    GreaterThan(Span),
    GreaterThanOrEqual(Span),
    Spaceship(Span),
}

/// Represents a PHP comparison operation
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ComparisonOperation {
    pub lhs: Expression,
    pub operator: ComparisonOperator,
    pub rhs: Expression,
}

impl ComparisonOperator {
    #[inline]
    pub fn is_equality(&self) -> bool {
        matches!(
            self,
            Self::Equal(_)
                | Self::NotEqual(_)
                | Self::Identical(_)
                | Self::NotIdentical(_)
                | Self::AngledNotEqual(_)
                | Self::Spaceship(_)
        )
    }

    #[inline]
    pub fn is_equal(&self) -> bool {
        matches!(self, Self::Equal(_))
    }

    #[inline]
    pub fn is_not_equal(&self) -> bool {
        matches!(self, Self::NotEqual(_))
    }

    #[inline]
    pub fn is_identical(&self) -> bool {
        matches!(self, Self::Identical(_))
    }

    #[inline]
    pub fn is_not_identical(&self) -> bool {
        matches!(self, Self::NotIdentical(_))
    }

    #[inline]
    pub fn is_angled_not_equal(&self) -> bool {
        matches!(self, Self::AngledNotEqual(_))
    }

    #[inline]
    pub fn is_less_than(&self) -> bool {
        matches!(self, Self::LessThan(_))
    }

    #[inline]
    pub fn is_less_than_or_equal(&self) -> bool {
        matches!(self, Self::LessThanOrEqual(_))
    }

    #[inline]
    pub fn is_greater_than(&self) -> bool {
        matches!(self, Self::GreaterThan(_))
    }

    #[inline]
    pub fn is_greater_than_or_equal(&self) -> bool {
        matches!(self, Self::GreaterThanOrEqual(_))
    }

    #[inline]
    pub fn is_spaceship(&self) -> bool {
        matches!(self, Self::Spaceship(_))
    }

    #[inline]
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Equal(_), Self::Equal(_))
            | (Self::NotEqual(_), Self::NotEqual(_))
            | (Self::Identical(_), Self::Identical(_))
            | (Self::NotIdentical(_), Self::NotIdentical(_))
            | (Self::AngledNotEqual(_), Self::AngledNotEqual(_))
            | (Self::LessThan(_), Self::LessThan(_))
            | (Self::LessThanOrEqual(_), Self::LessThanOrEqual(_))
            | (Self::GreaterThan(_), Self::GreaterThan(_))
            | (Self::GreaterThanOrEqual(_), Self::GreaterThanOrEqual(_))
            | (Self::Spaceship(_), Self::Spaceship(_)) => true,
            _ => false,
        }
    }
}

impl GetPrecedence for ComparisonOperator {
    fn precedence(&self) -> Precedence {
        match &self {
            Self::Equal(_)
            | Self::NotEqual(_)
            | Self::Identical(_)
            | Self::NotIdentical(_)
            | Self::AngledNotEqual(_)
            | Self::Spaceship(_) => Precedence::Equality,
            Self::LessThan(_) | Self::LessThanOrEqual(_) | Self::GreaterThan(_) | Self::GreaterThanOrEqual(_) => {
                Precedence::Comparison
            }
        }
    }
}

impl HasSpan for ComparisonOperator {
    fn span(&self) -> Span {
        match self {
            Self::Equal(span) => *span,
            Self::NotEqual(span) => *span,
            Self::Identical(span) => *span,
            Self::NotIdentical(span) => *span,
            Self::AngledNotEqual(span) => *span,
            Self::LessThan(span) => *span,
            Self::LessThanOrEqual(span) => *span,
            Self::GreaterThan(span) => *span,
            Self::GreaterThanOrEqual(span) => *span,
            Self::Spaceship(span) => *span,
        }
    }
}

impl HasSpan for ComparisonOperation {
    fn span(&self) -> Span {
        self.lhs.span().join(self.rhs.span())
    }
}
