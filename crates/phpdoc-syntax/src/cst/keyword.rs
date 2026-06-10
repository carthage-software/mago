use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;

use crate::token::Token;
use crate::token::TokenKind;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Keyword<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

impl HasSpan for Keyword<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> Keyword<'arena> {
    #[inline]
    #[must_use]
    pub fn from_token(token: Token<'arena>, file_id: FileId) -> Self {
        debug_assert_eq!(token.kind, TokenKind::Identifier, "expected an identifier token for a keyword");

        Keyword { span: token.span_for(file_id), value: token.value }
    }
}

impl std::fmt::Display for Keyword<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&String::from_utf8_lossy(self.value))
    }
}
