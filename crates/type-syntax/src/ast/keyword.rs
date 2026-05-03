use serde::Serialize;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;

use crate::token::TypeToken;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Keyword<'arena> {
    pub span: Span,
    pub value: &'arena str,
}

impl HasSpan for Keyword<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> Keyword<'arena> {
    /// Creates a Keyword from a TypeToken and file_id.
    #[inline]
    #[must_use]
    pub fn from_token(token: TypeToken<'arena>, file_id: FileId) -> Self {
        debug_assert!(
            token.kind.is_keyword()
                || (token.kind.is_identifier() && token.value.to_ascii_lowercase().ends_with("closure")),
            "Expected a keyword or identifier, found: {:?} ( `{}` )",
            token.kind,
            token.value
        );

        Keyword { span: token.span_for(file_id), value: token.value }
    }
}

impl std::fmt::Display for Keyword<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
