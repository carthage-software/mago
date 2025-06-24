use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::token::TypeToken;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Keyword<'input> {
    pub span: Span,
    pub value: &'input str,
}

impl HasSpan for Keyword<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'input> From<TypeToken<'input>> for Keyword<'input> {
    #[inline]
    fn from(token: TypeToken<'input>) -> Self {
        debug_assert!(
            token.kind.is_keyword()
                || (token.kind.is_identifier() && token.value.to_ascii_lowercase().ends_with("closure")),
            "Expected a keyword or identifier, found: {:?} ( `{}` )",
            token.kind,
            token.value
        );

        Keyword { span: token.span, value: token.value }
    }
}

impl std::fmt::Display for Keyword<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
