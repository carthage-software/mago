use mago_interner::ThreadedInterner;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;
use mago_token::GetPrecedence;
use mago_token::Precedence;

use crate::Keyword;
use crate::ast::expression::Expression;

/// Represents a PHP binary operator.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum BinaryOperator {
    Addition(Span),           // `+`
    Subtraction(Span),        // `-`
    Multiplication(Span),     // `*`
    Division(Span),           // `/`
    Modulo(Span),             // `%`
    Exponentiation(Span),     // `**`
    BitwiseAnd(Span),         // `&`
    BitwiseOr(Span),          // `|`
    BitwiseXor(Span),         // `^`
    LeftShift(Span),          // `<<`
    RightShift(Span),         // `>>`
    NullCoalesce(Span),       // `??`
    Equal(Span),              // `==`
    NotEqual(Span),           // `!=`
    Identical(Span),          // `===`
    NotIdentical(Span),       // `!==`
    AngledNotEqual(Span),     // `<>`
    LessThan(Span),           // `<`
    LessThanOrEqual(Span),    // `<=`
    GreaterThan(Span),        // `>`
    GreaterThanOrEqual(Span), // `>=`
    Spaceship(Span),          // `<=>`
    StringConcat(Span),       // `.`
    Instanceof(Keyword),      // `instanceof`
    And(Span),                // `&&`
    Or(Span),                 // `||`
    LowAnd(Keyword),          // `and`
    LowOr(Keyword),           // `or`
    LowXor(Keyword),          // `xor`
    Elvis(Span),              // `?:`
}

/// Represents a PHP binary operation.
///
/// A binary operation is an operation that takes two operands, a left-hand side and a right-hand side.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Binary {
    pub lhs: Box<Expression>,
    pub operator: BinaryOperator,
    pub rhs: Box<Expression>,
}

impl BinaryOperator {
    #[inline(always)]
    pub const fn is_constant(&self) -> bool {
        !matches!(self, Self::Elvis(_) | Self::Instanceof(_))
    }

    #[inline(always)]
    pub const fn is_multiplicative(&self) -> bool {
        matches!(self, Self::Multiplication(_) | Self::Division(_) | Self::Modulo(_))
    }

    #[inline(always)]
    pub const fn is_additive(&self) -> bool {
        matches!(self, Self::Addition(_) | Self::Subtraction(_))
    }

    #[inline(always)]
    pub const fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Self::Addition(_)
                | Self::Subtraction(_)
                | Self::Multiplication(_)
                | Self::Division(_)
                | Self::Modulo(_)
                | Self::Exponentiation(_)
        )
    }

    #[inline(always)]
    pub const fn is_bit_shift(&self) -> bool {
        matches!(self, Self::LeftShift(_) | Self::RightShift(_))
    }

    #[inline(always)]
    pub const fn is_bitwise(&self) -> bool {
        matches!(
            self,
            Self::BitwiseAnd(_) | Self::BitwiseOr(_) | Self::BitwiseXor(_) | Self::LeftShift(_) | Self::RightShift(_)
        )
    }

    #[inline(always)]
    pub const fn is_equality(&self) -> bool {
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

    #[inline(always)]
    pub const fn is_comparison(&self) -> bool {
        matches!(
            self,
            Self::Equal(_)
                | Self::NotEqual(_)
                | Self::Identical(_)
                | Self::NotIdentical(_)
                | Self::AngledNotEqual(_)
                | Self::LessThan(_)
                | Self::LessThanOrEqual(_)
                | Self::GreaterThan(_)
                | Self::GreaterThanOrEqual(_)
                | Self::Spaceship(_)
        )
    }

    #[inline(always)]
    pub const fn is_logical(&self) -> bool {
        matches!(self, Self::And(_) | Self::Or(_) | Self::LowAnd(_) | Self::LowOr(_) | Self::LowXor(_))
    }

    #[inline(always)]
    pub const fn is_concatenation(&self) -> bool {
        matches!(self, Self::StringConcat(_))
    }

    #[inline(always)]
    pub const fn is_null_coalesce(&self) -> bool {
        matches!(self, Self::NullCoalesce(_))
    }

    #[inline(always)]
    pub const fn is_elvis(&self) -> bool {
        matches!(self, Self::Elvis(_))
    }

    #[inline]
    pub fn as_str<'a>(&self, interner: &'a ThreadedInterner) -> &'a str {
        match self {
            Self::Addition(_) => "+",
            Self::Subtraction(_) => "-",
            Self::Multiplication(_) => "*",
            Self::Division(_) => "/",
            Self::Modulo(_) => "%",
            Self::Exponentiation(_) => "**",
            Self::BitwiseAnd(_) => "&",
            Self::BitwiseOr(_) => "|",
            Self::BitwiseXor(_) => "^",
            Self::LeftShift(_) => "<<",
            Self::RightShift(_) => ">>",
            Self::NullCoalesce(_) => "??",
            Self::Equal(_) => "==",
            Self::NotEqual(_) => "!=",
            Self::Identical(_) => "===",
            Self::NotIdentical(_) => "!==",
            Self::AngledNotEqual(_) => "<>",
            Self::LessThan(_) => "<",
            Self::LessThanOrEqual(_) => "<=",
            Self::GreaterThan(_) => ">",
            Self::GreaterThanOrEqual(_) => ">=",
            Self::Spaceship(_) => "<=>",
            Self::StringConcat(_) => ".",
            Self::And(_) => "&&",
            Self::Or(_) => "||",
            Self::Elvis(_) => "?:",
            Self::Instanceof(keyword) => interner.lookup(&keyword.value),
            Self::LowAnd(keyword) => interner.lookup(&keyword.value),
            Self::LowOr(keyword) => interner.lookup(&keyword.value),
            Self::LowXor(keyword) => interner.lookup(&keyword.value),
        }
    }

    #[inline]
    pub fn is_same_as(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Addition(_), Self::Addition(_))
                | (Self::Subtraction(_), Self::Subtraction(_))
                | (Self::Multiplication(_), Self::Multiplication(_))
                | (Self::Division(_), Self::Division(_))
                | (Self::Modulo(_), Self::Modulo(_))
                | (Self::Exponentiation(_), Self::Exponentiation(_))
                | (Self::BitwiseAnd(_), Self::BitwiseAnd(_))
                | (Self::BitwiseOr(_), Self::BitwiseOr(_))
                | (Self::BitwiseXor(_), Self::BitwiseXor(_))
                | (Self::LeftShift(_), Self::LeftShift(_))
                | (Self::RightShift(_), Self::RightShift(_))
                | (Self::NullCoalesce(_), Self::NullCoalesce(_))
                | (Self::Equal(_), Self::Equal(_))
                | (Self::NotEqual(_), Self::NotEqual(_))
                | (Self::Identical(_), Self::Identical(_))
                | (Self::NotIdentical(_), Self::NotIdentical(_))
                | (Self::AngledNotEqual(_), Self::AngledNotEqual(_))
                | (Self::LessThan(_), Self::LessThan(_))
                | (Self::LessThanOrEqual(_), Self::LessThanOrEqual(_))
                | (Self::GreaterThan(_), Self::GreaterThan(_))
                | (Self::GreaterThanOrEqual(_), Self::GreaterThanOrEqual(_))
                | (Self::Spaceship(_), Self::Spaceship(_))
                | (Self::StringConcat(_), Self::StringConcat(_))
                | (Self::Instanceof(_), Self::Instanceof(_))
                | (Self::And(_), Self::And(_))
                | (Self::Or(_), Self::Or(_))
                | (Self::LowAnd(_), Self::LowAnd(_))
                | (Self::LowOr(_), Self::LowOr(_))
                | (Self::LowXor(_), Self::LowXor(_))
                | (Self::Elvis(_), Self::Elvis(_))
        )
    }
}

impl GetPrecedence for BinaryOperator {
    #[inline]
    fn precedence(&self) -> Precedence {
        match self {
            Self::Addition(_) | Self::Subtraction(_) => Precedence::AddSub,
            Self::Multiplication(_) | Self::Division(_) | Self::Modulo(_) => Precedence::MulDivMod,
            Self::Exponentiation(_) => Precedence::Pow,
            Self::BitwiseAnd(_) => Precedence::BitwiseAnd,
            Self::BitwiseOr(_) => Precedence::BitwiseOr,
            Self::BitwiseXor(_) => Precedence::BitwiseXor,
            Self::LeftShift(_) | Self::RightShift(_) => Precedence::BitShift,
            Self::NullCoalesce(_) => Precedence::NullCoalesce,
            Self::Equal(_)
            | Self::NotEqual(_)
            | Self::Identical(_)
            | Self::NotIdentical(_)
            | Self::AngledNotEqual(_)
            | Self::Spaceship(_) => Precedence::Equality,
            Self::LessThan(_) | Self::LessThanOrEqual(_) | Self::GreaterThan(_) | Self::GreaterThanOrEqual(_) => {
                Precedence::Comparison
            }
            Self::StringConcat(_) => Precedence::Concat,
            Self::And(_) => Precedence::And,
            Self::Or(_) => Precedence::Or,
            Self::LowAnd(_) => Precedence::KeyAnd,
            Self::LowOr(_) => Precedence::KeyOr,
            Self::LowXor(_) => Precedence::KeyXor,
            Self::Instanceof(_) => Precedence::Instanceof,
            Self::Elvis(_) => Precedence::ElvisOrConditional,
        }
    }
}

impl HasSpan for BinaryOperator {
    fn span(&self) -> Span {
        match self {
            Self::Addition(span) => *span,
            Self::Subtraction(span) => *span,
            Self::Multiplication(span) => *span,
            Self::Division(span) => *span,
            Self::Modulo(span) => *span,
            Self::Exponentiation(span) => *span,
            Self::BitwiseAnd(span) => *span,
            Self::BitwiseOr(span) => *span,
            Self::BitwiseXor(span) => *span,
            Self::LeftShift(span) => *span,
            Self::RightShift(span) => *span,
            Self::NullCoalesce(span) => *span,
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
            Self::StringConcat(span) => *span,
            Self::Instanceof(keyword) => keyword.span(),
            Self::And(span) => *span,
            Self::Or(span) => *span,
            Self::LowAnd(keyword) => keyword.span(),
            Self::LowOr(keyword) => keyword.span(),
            Self::LowXor(keyword) => keyword.span(),
            Self::Elvis(span) => *span,
        }
    }
}

impl HasSpan for Binary {
    fn span(&self) -> Span {
        self.lhs.span().join(self.rhs.span())
    }
}
