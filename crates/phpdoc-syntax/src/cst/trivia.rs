use strum::Display;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;

use crate::token::Token;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum TriviaKind {
    OpeningMarker,
    ClosingMarker,
    Trailing,
    Asterisk,
    Whitespace,
    LineComment,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Trivia<'arena> {
    pub kind: TriviaKind,
    pub span: Span,
    pub value: &'arena [u8],
}

impl<'arena> Trivia<'arena> {
    #[inline]
    #[must_use]
    pub fn from_token(kind: TriviaKind, token: Token<'arena>, file_id: FileId) -> Self {
        Trivia { kind, span: token.span_for(file_id), value: token.value }
    }
}

impl HasSpan for Trivia<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
