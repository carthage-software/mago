//! Token types produced by the surface-grammar lexer.
//!
//! Each token carries its source slice and start position so parse errors can point at
//! precise offsets. The token kind is a discriminator; numeric values are parsed lazily
//! by the parser rather than eagerly at lex time.

use mago_span::Position;

/// Kind of a surface-grammar token.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SurfaceTokenKind {
    /// Run of whitespace (space, tab, newline, carriage return).
    Whitespace,
    /// `// …` line comment.
    LineComment,
    /// `` ` … ` `` code-snippet delimiter body (value excludes the backticks).
    Backtick,
    /// Unterminated backtick body (EOF reached before closing `` ` ``).
    UnterminatedBacktick,
    /// `^name` metavariable reference (value includes the leading `^`).
    Variable,
    /// `"…"` string literal (value includes both quotes).
    String,
    /// Unterminated `"…` string literal (EOF reached before closing `"`).
    UnterminatedString,
    /// Integer literal (digits only).
    Integer,
    /// Floating-point literal (digits with a `.`).
    Float,
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `,`
    Comma,
    /// `<:` subtype operator.
    Subtype,
    /// `=>` rewrite operator.
    Rewrite,
    /// `!` prefix negation.
    Bang,
    /// `not` keyword.
    KwNot,
    /// `contains` keyword.
    KwContains,
    /// `within` keyword.
    KwWithin,
    /// `maybe` keyword.
    KwMaybe,
    /// `bubble` keyword.
    KwBubble,
    /// `where` keyword.
    KwWhere,
    /// `or` keyword.
    KwOr,
    /// `and` keyword.
    KwAnd,
    /// `as` keyword.
    KwAs,
    /// `undefined` keyword.
    KwUndefined,
    /// `true` boolean literal.
    True,
    /// `false` boolean literal.
    False,
    /// Bare identifier (no keyword match).
    Identifier,
    /// Unrecognized byte (produced instead of panicking so the parser can report it).
    Unknown,
}

impl SurfaceTokenKind {
    /// Returns `true` if this token should be skipped by the parser (whitespace, comments).
    #[inline]
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::Whitespace | Self::LineComment)
    }
}

/// A surface-grammar token: a kind plus the exact source slice and start position.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SurfaceToken<'input> {
    pub kind: SurfaceTokenKind,
    pub value: &'input str,
    pub start: Position,
}

impl<'input> SurfaceToken<'input> {
    #[inline]
    #[must_use]
    pub const fn new(kind: SurfaceTokenKind, value: &'input str, start: Position) -> Self {
        Self { kind, value, start }
    }
}
