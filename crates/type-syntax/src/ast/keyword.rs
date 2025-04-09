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
        Keyword { span: token.span, value: token.value }
    }
}
