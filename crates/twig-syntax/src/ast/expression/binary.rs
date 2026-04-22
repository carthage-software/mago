use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum BinaryOperator<'arena> {
    Addition(Span),
    Subtraction(Span),
    Multiplication(Span),
    Division(Span),
    FloorDivision(Span),
    Modulo(Span),
    Exponentiation(Span),
    StringConcat(Span),
    Range(Span),
    Equal(Span),
    NotEqual(Span),
    LessThan(Span),
    LessThanOrEqual(Span),
    GreaterThan(Span),
    GreaterThanOrEqual(Span),
    Spaceship(Span),
    Identical(Span),
    NotIdentical(Span),
    And(Keyword<'arena>),
    Or(Keyword<'arena>),
    Xor(Keyword<'arena>),
    BitwiseAnd(Keyword<'arena>),
    BitwiseOr(Keyword<'arena>),
    BitwiseXor(Keyword<'arena>),
    NullCoalesce(Span),
    In(Keyword<'arena>),
    NotIn(Keyword<'arena>),
    Matches(Keyword<'arena>),
    StartsWith(Keyword<'arena>),
    EndsWith(Keyword<'arena>),
    HasSome(Keyword<'arena>),
    HasEvery(Keyword<'arena>),
    Assignment(Span),
}

impl BinaryOperator<'_> {
    #[inline]
    #[must_use]
    pub const fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::And(_)
                | Self::Or(_)
                | Self::Xor(_)
                | Self::BitwiseAnd(_)
                | Self::BitwiseOr(_)
                | Self::BitwiseXor(_)
                | Self::In(_)
                | Self::NotIn(_)
                | Self::Matches(_)
                | Self::StartsWith(_)
                | Self::EndsWith(_)
                | Self::HasSome(_)
                | Self::HasEvery(_),
        )
    }
}

impl HasSpan for BinaryOperator<'_> {
    fn span(&self) -> Span {
        match self {
            BinaryOperator::Addition(s)
            | BinaryOperator::Subtraction(s)
            | BinaryOperator::Multiplication(s)
            | BinaryOperator::Division(s)
            | BinaryOperator::FloorDivision(s)
            | BinaryOperator::Modulo(s)
            | BinaryOperator::Exponentiation(s)
            | BinaryOperator::StringConcat(s)
            | BinaryOperator::Range(s)
            | BinaryOperator::Equal(s)
            | BinaryOperator::NotEqual(s)
            | BinaryOperator::LessThan(s)
            | BinaryOperator::LessThanOrEqual(s)
            | BinaryOperator::GreaterThan(s)
            | BinaryOperator::GreaterThanOrEqual(s)
            | BinaryOperator::Spaceship(s)
            | BinaryOperator::Identical(s)
            | BinaryOperator::NotIdentical(s)
            | BinaryOperator::NullCoalesce(s)
            | BinaryOperator::Assignment(s) => *s,
            BinaryOperator::And(k)
            | BinaryOperator::Or(k)
            | BinaryOperator::Xor(k)
            | BinaryOperator::BitwiseAnd(k)
            | BinaryOperator::BitwiseOr(k)
            | BinaryOperator::BitwiseXor(k)
            | BinaryOperator::In(k)
            | BinaryOperator::NotIn(k)
            | BinaryOperator::Matches(k)
            | BinaryOperator::StartsWith(k)
            | BinaryOperator::EndsWith(k)
            | BinaryOperator::HasSome(k)
            | BinaryOperator::HasEvery(k) => k.span,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Binary<'arena> {
    pub lhs: &'arena Expression<'arena>,
    pub operator: BinaryOperator<'arena>,
    pub rhs: &'arena Expression<'arena>,
}

impl HasSpan for Binary<'_> {
    fn span(&self) -> Span {
        self.lhs.span().join(self.rhs.span())
    }
}
