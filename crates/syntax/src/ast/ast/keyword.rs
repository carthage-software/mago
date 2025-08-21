use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

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
