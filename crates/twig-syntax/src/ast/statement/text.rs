use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

/// Raw template text (including whitespace between tags).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Text<'arena> {
    pub value: &'arena str,
    pub span: Span,
}

impl HasSpan for Text<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
