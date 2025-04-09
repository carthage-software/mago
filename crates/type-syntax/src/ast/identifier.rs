use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::token::TypeToken;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Identifier<'input> {
    pub span: Span,
    pub value: &'input str,
}

impl HasSpan for Identifier<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'input> From<TypeToken<'input>> for Identifier<'input> {
    #[inline]
    fn from(token: TypeToken<'input>) -> Self {
        Identifier { span: token.span, value: token.value }
    }
}
