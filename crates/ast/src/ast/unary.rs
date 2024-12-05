use fennec_interner::ThreadedInterner;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_interner::StringIdentifier;
use fennec_span::HasSpan;
use fennec_span::Span;
use fennec_token::GetPrecedence;
use fennec_token::Precedence;

use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum UnaryPrefixOperator {
    ErrorControl(Span),                  // `@$expr`
    Reference(Span),                     // `&$expr`
    ArrayCast(Span, StringIdentifier),   // `(array) $expr`
    BoolCast(Span, StringIdentifier),    // `(bool) $expr`
    BooleanCast(Span, StringIdentifier), // `(boolean) $expr`
    DoubleCast(Span, StringIdentifier),  // `(double) $expr`
    RealCast(Span, StringIdentifier),    // `(real) $expr`
    FloatCast(Span, StringIdentifier),   // `(float) $expr`
    IntCast(Span, StringIdentifier),     // `(int) $expr`
    IntegerCast(Span, StringIdentifier), // `(integer) $expr`
    ObjectCast(Span, StringIdentifier),  // `(object) $expr`
    UnsetCast(Span, StringIdentifier),   // `(unset) $expr`
    StringCast(Span, StringIdentifier),  // `(string) $expr`
    BinaryCast(Span, StringIdentifier),  // `(binary) $expr`
    BitwiseNot(Span),                    // `~$expr`
    Not(Span),                           // `!$expr`
    PreIncrement(Span),                  // `++$expr`
    PreDecrement(Span),                  // `--$expr`
    Plus(Span),                          // `+$expr`
    Negation(Span),                      // `-$expr`
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum UnaryPostfixOperator {
    PostIncrement(Span), // `$expr++`
    PostDecrement(Span), // `$expr--`
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct UnaryPrefix {
    pub operator: UnaryPrefixOperator,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct UnaryPostfix {
    pub operand: Box<Expression>,
    pub operator: UnaryPostfixOperator,
}

impl UnaryPrefixOperator {
    #[inline]
    pub const fn is_constant(&self) -> bool {
        matches!(
            self,
            Self::BitwiseNot(_)
                | Self::Not(_)
                | Self::PreIncrement(_)
                | Self::PreDecrement(_)
                | Self::Plus(_)
                | Self::Negation(_)
        )
    }

    #[inline]
    pub const fn is_cast(&self) -> bool {
        matches!(
            self,
            Self::ArrayCast(_, _)
                | Self::BoolCast(_, _)
                | Self::BooleanCast(_, _)
                | Self::DoubleCast(_, _)
                | Self::RealCast(_, _)
                | Self::FloatCast(_, _)
                | Self::IntCast(_, _)
                | Self::IntegerCast(_, _)
                | Self::ObjectCast(_, _)
                | Self::UnsetCast(_, _)
                | Self::StringCast(_, _)
                | Self::BinaryCast(_, _)
        )
    }

    #[inline]
    pub const fn is_arithmetic(&self) -> bool {
        matches!(self, Self::Plus(_) | Self::Negation(_) | Self::PreIncrement(_) | Self::PreDecrement(_))
    }

    #[inline]
    pub fn as_str<'a>(&self, interner: &'a ThreadedInterner) -> &'a str {
        match self {
            UnaryPrefixOperator::ErrorControl(_) => "@",
            UnaryPrefixOperator::Reference(_) => "&",
            UnaryPrefixOperator::ArrayCast(_, value)
            | UnaryPrefixOperator::BoolCast(_, value)
            | UnaryPrefixOperator::BooleanCast(_, value)
            | UnaryPrefixOperator::DoubleCast(_, value)
            | UnaryPrefixOperator::RealCast(_, value)
            | UnaryPrefixOperator::FloatCast(_, value)
            | UnaryPrefixOperator::IntCast(_, value)
            | UnaryPrefixOperator::IntegerCast(_, value)
            | UnaryPrefixOperator::ObjectCast(_, value)
            | UnaryPrefixOperator::UnsetCast(_, value)
            | UnaryPrefixOperator::StringCast(_, value)
            | UnaryPrefixOperator::BinaryCast(_, value) => interner.lookup(value),
            UnaryPrefixOperator::BitwiseNot(_) => "~",
            UnaryPrefixOperator::Not(_) => "!",
            UnaryPrefixOperator::PreIncrement(_) => "++",
            UnaryPrefixOperator::PreDecrement(_) => "--",
            UnaryPrefixOperator::Plus(_) => "+",
            UnaryPrefixOperator::Negation(_) => "-",
        }
    }

    #[inline]
    pub const fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ErrorControl(_), Self::ErrorControl(_)) => true,
            (Self::Reference(_), Self::Reference(_)) => true,
            (Self::ArrayCast(_, _), Self::ArrayCast(_, _)) => true,
            (Self::BoolCast(_, _), Self::BoolCast(_, _)) => true,
            (Self::BooleanCast(_, _), Self::BooleanCast(_, _)) => true,
            (Self::DoubleCast(_, _), Self::DoubleCast(_, _)) => true,
            (Self::RealCast(_, _), Self::RealCast(_, _)) => true,
            (Self::FloatCast(_, _), Self::FloatCast(_, _)) => true,
            (Self::IntCast(_, _), Self::IntCast(_, _)) => true,
            (Self::IntegerCast(_, _), Self::IntegerCast(_, _)) => true,
            (Self::ObjectCast(_, _), Self::ObjectCast(_, _)) => true,
            (Self::UnsetCast(_, _), Self::UnsetCast(_, _)) => true,
            (Self::StringCast(_, _), Self::StringCast(_, _)) => true,
            (Self::BinaryCast(_, _), Self::BinaryCast(_, _)) => true,
            (Self::BitwiseNot(_), Self::BitwiseNot(_)) => true,
            (Self::Not(_), Self::Not(_)) => true,
            (Self::PreIncrement(_), Self::PreIncrement(_)) => true,
            (Self::PreDecrement(_), Self::PreDecrement(_)) => true,
            (Self::Plus(_), Self::Plus(_)) => true,
            (Self::Negation(_), Self::Negation(_)) => true,
            _ => false,
        }
    }
}

impl UnaryPostfixOperator {
    #[inline]
    pub const fn is_constant(&self) -> bool {
        match self {
            Self::PostIncrement(_) | Self::PostDecrement(_) => false,
        }
    }

    #[inline]
    pub const fn as_str<'a>(&self) -> &'a str {
        match self {
            UnaryPostfixOperator::PostIncrement(_) => "++",
            UnaryPostfixOperator::PostDecrement(_) => "--",
        }
    }

    #[inline]
    pub const fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PostIncrement(_), Self::PostIncrement(_)) => true,
            (Self::PostDecrement(_), Self::PostDecrement(_)) => true,
            _ => false,
        }
    }
}

impl GetPrecedence for UnaryPostfixOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::PostIncrement(_) | Self::PostDecrement(_) => Precedence::IncDec,
        }
    }
}

impl HasSpan for UnaryPrefixOperator {
    fn span(&self) -> Span {
        match self {
            Self::ErrorControl(span) => *span,
            Self::Reference(span) => *span,
            Self::ArrayCast(span, ..) => *span,
            Self::BoolCast(span, ..) => *span,
            Self::BooleanCast(span, ..) => *span,
            Self::DoubleCast(span, ..) => *span,
            Self::RealCast(span, ..) => *span,
            Self::FloatCast(span, ..) => *span,
            Self::IntCast(span, ..) => *span,
            Self::IntegerCast(span, ..) => *span,
            Self::ObjectCast(span, ..) => *span,
            Self::UnsetCast(span, ..) => *span,
            Self::StringCast(span, ..) => *span,
            Self::BinaryCast(span, ..) => *span,
            Self::BitwiseNot(span) => *span,
            Self::Not(span) => *span,
            Self::PreIncrement(span) => *span,
            Self::PreDecrement(span) => *span,
            Self::Plus(span) => *span,
            Self::Negation(span) => *span,
        }
    }
}

impl HasSpan for UnaryPostfixOperator {
    fn span(&self) -> Span {
        match self {
            Self::PostIncrement(span) => *span,
            Self::PostDecrement(span) => *span,
        }
    }
}

impl HasSpan for UnaryPrefix {
    fn span(&self) -> Span {
        self.operator.span().join(self.operand.span())
    }
}

impl HasSpan for UnaryPostfix {
    fn span(&self) -> Span {
        self.operand.span().join(self.operator.span())
    }
}
