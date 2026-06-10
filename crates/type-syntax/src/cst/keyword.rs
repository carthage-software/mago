use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;

use crate::token::TypeToken;

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
    /// Creates a Keyword from a TypeToken and file_id.
    #[inline]
    #[must_use]
    pub fn from_token(token: TypeToken<'arena>, file_id: FileId) -> Self {
        debug_assert!(
            token.kind.is_keyword()
                || (token.kind.is_identifier() && token.value.to_ascii_lowercase().ends_with(b"closure")),
            "Expected a keyword or identifier, found: {:?} ( `{}` )",
            token.kind,
            String::from_utf8_lossy(token.value),
        );

        Keyword { span: token.span_for(file_id), value: token.value }
    }
}

impl std::fmt::Display for Keyword<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&String::from_utf8_lossy(self.value))
    }
}
