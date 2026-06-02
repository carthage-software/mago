use serde::Serialize;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;

use crate::token::Token;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Text<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

impl<'arena> Text<'arena> {
    #[inline]
    #[must_use]
    pub const fn new(span: Span, value: &'arena [u8]) -> Self {
        Text { span, value }
    }

    #[inline]
    #[must_use]
    pub fn from_token(token: Token<'arena>, file_id: FileId) -> Self {
        Text { span: token.span_for(file_id), value: token.value }
    }
}

impl HasSpan for Text<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
