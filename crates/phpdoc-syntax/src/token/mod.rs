use strum::Display;

use mago_database::file::FileId;
use mago_span::HasPosition;
use mago_span::Position;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum TokenKind {
    OpeningMarker,
    ClosingMarker,
    Tag,
    Whitespace,
    LineComment,
    Pipe,
    Ampersand,
    Question,
    Bang,
    LeftParenthesis,
    RightParenthesis,
    LeftAngleBracket,
    RightAngleBracket,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    ColonColon,
    Arrow,
    DoubleArrow,
    Equals,
    Minus,
    Plus,
    Ellipsis,
    Asterisk,
    Backtick,
    LiteralInteger,
    LiteralFloat,
    SingleQuotedString,
    DoubleQuotedString,
    PartialString,
    Identifier,
    Variable,
    ThisVariable,
    Other,
}

impl TokenKind {
    #[inline]
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, TokenKind::Whitespace | TokenKind::LineComment)
    }

    #[inline]
    #[must_use]
    pub const fn is_string_literal(&self) -> bool {
        matches!(self, TokenKind::SingleQuotedString | TokenKind::DoubleQuotedString)
    }

    #[inline]
    #[must_use]
    pub const fn is_number_literal(&self) -> bool {
        matches!(self, TokenKind::LiteralInteger | TokenKind::LiteralFloat)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Token<'arena> {
    pub kind: TokenKind,
    pub start: Position,
    pub value: &'arena [u8],
}

impl<'arena> Token<'arena> {
    #[inline]
    #[must_use]
    pub const fn new(kind: TokenKind, value: &'arena [u8], start: Position) -> Self {
        Self { kind, start, value }
    }

    #[inline]
    #[must_use]
    pub const fn span_for(&self, file_id: FileId) -> Span {
        Span::new(file_id, self.start, Position::new(self.start.offset + self.value.len() as u32))
    }
}

impl HasPosition for Token<'_> {
    #[inline]
    fn position(&self) -> Position {
        self.start
    }
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.kind, String::from_utf8_lossy(self.value))
    }
}
