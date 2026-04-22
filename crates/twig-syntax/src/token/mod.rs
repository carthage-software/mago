use serde::Serialize;
use strum::Display;

use mago_database::file::FileId;
use mago_span::Position;
use mago_span::Span;

/// Associativity classification for infix operators.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum Associativity {
    NonAssociative,
    Left,
    Right,
}

/// Precedence levels for Twig expression operators, ordered from lowest to
/// highest binding.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum Precedence {
    Lowest,
    Elvis,
    Conditional,
    Or,
    Xor,
    And,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Comparison,
    Range,
    AddSub,
    Concat,
    MulDivMod,
    Unary,
    Pow,
    NullCoalesce,
    Filter,
    Test,
    Access,
    Highest,
}

impl Precedence {
    /// The precedence of `kind` when used as a left-binding infix operator, or
    /// [`Precedence::Lowest`] if `kind` does not participate in infix parsing.
    #[inline]
    #[must_use]
    pub const fn infix(kind: TwigTokenKind) -> Precedence {
        match kind {
            TwigTokenKind::QuestionColon => Precedence::Elvis,
            TwigTokenKind::Question => Precedence::Conditional,
            TwigTokenKind::Or => Precedence::Or,
            TwigTokenKind::Xor => Precedence::Xor,
            TwigTokenKind::And => Precedence::And,
            TwigTokenKind::BOr => Precedence::BitwiseOr,
            TwigTokenKind::BXor => Precedence::BitwiseXor,
            TwigTokenKind::BAnd => Precedence::BitwiseAnd,
            TwigTokenKind::EqualEqual
            | TwigTokenKind::BangEqual
            | TwigTokenKind::EqualEqualEqual
            | TwigTokenKind::BangEqualEqual
            | TwigTokenKind::LessThan
            | TwigTokenKind::GreaterThan
            | TwigTokenKind::LessThanEqual
            | TwigTokenKind::GreaterThanEqual
            | TwigTokenKind::Spaceship
            | TwigTokenKind::In
            | TwigTokenKind::NotIn
            | TwigTokenKind::Matches
            | TwigTokenKind::StartsWith
            | TwigTokenKind::EndsWith
            | TwigTokenKind::HasSome
            | TwigTokenKind::HasEvery => Precedence::Comparison,
            TwigTokenKind::DotDot => Precedence::Range,
            TwigTokenKind::Plus | TwigTokenKind::Minus => Precedence::AddSub,
            TwigTokenKind::Tilde => Precedence::Concat,
            TwigTokenKind::Asterisk | TwigTokenKind::Slash | TwigTokenKind::SlashSlash | TwigTokenKind::Percent => {
                Precedence::MulDivMod
            }
            TwigTokenKind::AsteriskAsterisk => Precedence::Pow,
            TwigTokenKind::QuestionQuestion => Precedence::NullCoalesce,
            TwigTokenKind::Pipe => Precedence::Filter,
            TwigTokenKind::Is => Precedence::Test,
            TwigTokenKind::Dot | TwigTokenKind::QuestionDot | TwigTokenKind::LeftBracket | TwigTokenKind::LeftParen => {
                Precedence::Access
            }
            _ => Precedence::Lowest,
        }
    }

    /// Associativity of `kind` when used as an infix operator.
    #[inline]
    #[must_use]
    pub const fn associativity(kind: TwigTokenKind) -> Associativity {
        match kind {
            TwigTokenKind::AsteriskAsterisk | TwigTokenKind::QuestionQuestion | TwigTokenKind::QuestionColon => {
                Associativity::Right
            }
            TwigTokenKind::Plus
            | TwigTokenKind::Minus
            | TwigTokenKind::Asterisk
            | TwigTokenKind::Slash
            | TwigTokenKind::SlashSlash
            | TwigTokenKind::Percent
            | TwigTokenKind::Tilde
            | TwigTokenKind::DotDot
            | TwigTokenKind::And
            | TwigTokenKind::Or
            | TwigTokenKind::Xor
            | TwigTokenKind::BAnd
            | TwigTokenKind::BOr
            | TwigTokenKind::BXor
            | TwigTokenKind::EqualEqual
            | TwigTokenKind::BangEqual
            | TwigTokenKind::EqualEqualEqual
            | TwigTokenKind::BangEqualEqual
            | TwigTokenKind::LessThan
            | TwigTokenKind::GreaterThan
            | TwigTokenKind::LessThanEqual
            | TwigTokenKind::GreaterThanEqual
            | TwigTokenKind::Spaceship
            | TwigTokenKind::In
            | TwigTokenKind::NotIn
            | TwigTokenKind::Matches
            | TwigTokenKind::StartsWith
            | TwigTokenKind::EndsWith
            | TwigTokenKind::HasSome
            | TwigTokenKind::HasEvery
            | TwigTokenKind::Pipe
            | TwigTokenKind::Dot
            | TwigTokenKind::QuestionDot
            | TwigTokenKind::LeftBracket
            | TwigTokenKind::LeftParen => Associativity::Left,
            _ => Associativity::NonAssociative,
        }
    }
}

/// Trait satisfied by types that carry a [`Precedence`] directly.
pub trait GetPrecedence {
    fn precedence(&self) -> Precedence;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum TwigTokenKind {
    RawText,
    VerbatimText,
    OpenBlock,
    OpenBlockDash,
    OpenBlockTilde,
    CloseBlock,
    CloseBlockDash,
    CloseBlockTilde,
    OpenVariable,
    OpenVariableDash,
    OpenVariableTilde,
    CloseVariable,
    CloseVariableDash,
    CloseVariableTilde,
    Comment,
    InlineComment,
    Name,
    Number,
    StringSingleQuoted,
    StringDoubleQuoted,
    DoubleQuoteStart,
    DoubleQuoteEnd,
    StringPart,
    InterpolationStart,
    InterpolationEnd,
    Whitespace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Dot,
    Question,
    Pipe,
    Equal,
    FatArrow,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    AsteriskAsterisk,
    SlashSlash,
    EqualEqual,
    BangEqual,
    EqualEqualEqual,
    BangEqualEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Spaceship,
    Tilde,
    DotDot,
    DotDotDot,
    QuestionQuestion,
    QuestionColon,
    QuestionDot,
    And,
    Or,
    Xor,
    BAnd,
    BOr,
    BXor,
    In,
    NotIn,
    Is,
    Not,
    Matches,
    StartsWith,
    EndsWith,
    HasSome,
    HasEvery,
    SameAs,
    DivisibleBy,
}

impl TwigTokenKind {
    #[inline]
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::Whitespace | Self::InlineComment | Self::Comment)
    }

    #[inline]
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::StringSingleQuoted | Self::StringDoubleQuoted)
    }

    #[inline]
    #[must_use]
    pub const fn is_open_block(self) -> bool {
        matches!(self, Self::OpenBlock | Self::OpenBlockDash | Self::OpenBlockTilde)
    }

    #[inline]
    #[must_use]
    pub const fn is_close_block(self) -> bool {
        matches!(self, Self::CloseBlock | Self::CloseBlockDash | Self::CloseBlockTilde)
    }

    #[inline]
    #[must_use]
    pub const fn is_open_variable(self) -> bool {
        matches!(self, Self::OpenVariable | Self::OpenVariableDash | Self::OpenVariableTilde)
    }

    #[inline]
    #[must_use]
    pub const fn is_close_variable(self) -> bool {
        matches!(self, Self::CloseVariable | Self::CloseVariableDash | Self::CloseVariableTilde)
    }
}

/// A Twig token.
///
/// Stores `{ kind, start, value }`. The end position and full [`Span`] are
/// derived on demand from `start` and `value.len()`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TwigToken<'input> {
    pub kind: TwigTokenKind,
    pub start: Position,
    pub value: &'input str,
}

impl mago_span::HasPosition for TwigToken<'_> {
    #[inline]
    fn position(&self) -> Position {
        self.start
    }
}

impl<'input> TwigToken<'input> {
    #[inline]
    #[must_use]
    pub const fn new(kind: TwigTokenKind, value: &'input str, start: Position) -> Self {
        Self { kind, start, value }
    }

    /// End position (one past the last byte).
    #[inline]
    #[must_use]
    pub const fn end(&self) -> Position {
        Position::new(self.start.offset + self.value.len() as u32)
    }

    /// Full span of this token, anchored to `file_id`.
    #[inline]
    #[must_use]
    pub const fn span_for(&self, file_id: FileId) -> Span {
        Span::new(file_id, self.start, self.end())
    }

    #[inline]
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        self.kind.is_trivia()
    }
}

impl std::fmt::Display for TwigToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}({})", self.kind, self.value)
    }
}
