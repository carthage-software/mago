use serde::Serialize;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Span;

use crate::token::TypeToken;
use crate::token::TypeTokenKind;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct VariableType<'arena> {
    pub span: Span,
    pub value: &'arena str,
}

impl HasSpan for VariableType<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> VariableType<'arena> {
    /// Creates a VariableType from a TypeToken and file_id.
    #[inline]
    pub fn from_token(token: TypeToken<'arena>, file_id: FileId) -> Self {
        debug_assert_eq!(token.kind, TypeTokenKind::Variable);

        VariableType { span: token.span_for(file_id), value: token.value }
    }
}

impl std::fmt::Display for VariableType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
