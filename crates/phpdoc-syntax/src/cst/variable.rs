use serde::Serialize;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;

use crate::token::Token;
use crate::token::TokenKind;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Variable<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

impl HasSpan for Variable<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> Variable<'arena> {
    #[inline]
    #[must_use]
    pub fn from_token(token: Token<'arena>, file_id: FileId) -> Self {
        debug_assert!(matches!(token.kind, TokenKind::Variable | TokenKind::ThisVariable), "expected a Variable token");

        Variable { span: token.span_for(file_id), value: token.value }
    }
}

impl std::fmt::Display for Variable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&String::from_utf8_lossy(self.value))
    }
}
