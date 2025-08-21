use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::token::GetPrecedence;
use crate::token::Precedence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum UnaryPrefixOperator<'arena> {
    ErrorControl(Span),             // `@$expr`
    Reference(Span),                // `&$expr`
    ArrayCast(Span, &'arena str),   // `(array) $expr`
    BoolCast(Span, &'arena str),    // `(bool) $expr`
    BooleanCast(Span, &'arena str), // `(boolean) $expr`
    DoubleCast(Span, &'arena str),  // `(double) $expr`
    RealCast(Span, &'arena str),    // `(real) $expr`
    FloatCast(Span, &'arena str),   // `(float) $expr`
    IntCast(Span, &'arena str),     // `(int) $expr`
    IntegerCast(Span, &'arena str), // `(integer) $expr`
    ObjectCast(Span, &'arena str),  // `(object) $expr`
    UnsetCast(Span, &'arena str),   // `(unset) $expr`
    StringCast(Span, &'arena str),  // `(string) $expr`
    BinaryCast(Span, &'arena str),  // `(binary) $expr`
    VoidCast(Span, &'arena str),    // `(void) $expr`
    BitwiseNot(Span),               // `~$expr`
    Not(Span),                      // `!$expr`
    PreIncrement(Span),             // `++$expr`
    PreDecrement(Span),             // `--$expr`
    Plus(Span),                     // `+$expr`
    Negation(Span),                 // `-$expr`
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum UnaryPostfixOperator {
    PostIncrement(Span), // `$expr++`
    PostDecrement(Span), // `$expr--`
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UnaryPrefix<'arena> {
    pub operator: UnaryPrefixOperator<'arena>,
    pub operand: &'arena Expression<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UnaryPostfix<'arena> {
    pub operand: &'arena Expression<'arena>,
    pub operator: UnaryPostfixOperator,
}

impl<'arena> UnaryPrefixOperator<'arena> {
    #[inline]
    pub const fn is_error_control(&self) -> bool {
        matches!(self, Self::ErrorControl(_))
    }

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
                | Self::VoidCast(_, _)
        )
    }

    #[inline]
    pub const fn is_arithmetic(&self) -> bool {
        matches!(self, Self::Plus(_) | Self::Negation(_) | Self::PreIncrement(_) | Self::PreDecrement(_))
    }

    #[inline]
    pub const fn is_increment_or_decrement(&self) -> bool {
        matches!(self, Self::PreIncrement(_) | Self::PreDecrement(_))
    }

    #[inline]
    pub const fn is_not(&self) -> bool {
        matches!(self, Self::Not(_))
    }

    #[inline]
    pub fn as_str(&self) -> &'arena str {
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
            | UnaryPrefixOperator::BinaryCast(_, value)
            | UnaryPrefixOperator::VoidCast(_, value) => value,
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
        matches!(
            (self, other),
            (Self::ErrorControl(_), Self::ErrorControl(_))
                | (Self::Reference(_), Self::Reference(_))
                | (Self::ArrayCast(_, _), Self::ArrayCast(_, _))
                | (Self::BoolCast(_, _), Self::BoolCast(_, _))
                | (Self::BooleanCast(_, _), Self::BooleanCast(_, _))
                | (Self::DoubleCast(_, _), Self::DoubleCast(_, _))
                | (Self::RealCast(_, _), Self::RealCast(_, _))
                | (Self::FloatCast(_, _), Self::FloatCast(_, _))
                | (Self::IntCast(_, _), Self::IntCast(_, _))
                | (Self::IntegerCast(_, _), Self::IntegerCast(_, _))
                | (Self::ObjectCast(_, _), Self::ObjectCast(_, _))
                | (Self::UnsetCast(_, _), Self::UnsetCast(_, _))
                | (Self::StringCast(_, _), Self::StringCast(_, _))
                | (Self::BinaryCast(_, _), Self::BinaryCast(_, _))
                | (Self::VoidCast(_, _), Self::VoidCast(_, _))
                | (Self::BitwiseNot(_), Self::BitwiseNot(_))
                | (Self::Not(_), Self::Not(_))
                | (Self::PreIncrement(_), Self::PreIncrement(_))
                | (Self::PreDecrement(_), Self::PreDecrement(_))
                | (Self::Plus(_), Self::Plus(_))
                | (Self::Negation(_), Self::Negation(_))
        )
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
        matches!(
            (self, other),
            (Self::PostIncrement(_), Self::PostIncrement(_)) | (Self::PostDecrement(_), Self::PostDecrement(_))
        )
    }
}

impl GetPrecedence for UnaryPostfixOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::PostIncrement(_) | Self::PostDecrement(_) => Precedence::IncDec,
        }
    }
}

impl HasSpan for UnaryPrefixOperator<'_> {
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
            Self::VoidCast(span, ..) => *span,
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

impl HasSpan for UnaryPrefix<'_> {
    fn span(&self) -> Span {
        self.operator.span().join(self.operand.span())
    }
}

impl HasSpan for UnaryPostfix<'_> {
    fn span(&self) -> Span {
        self.operand.span().join(self.operator.span())
    }
}
