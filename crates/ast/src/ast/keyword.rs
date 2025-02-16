use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Keyword {
    pub span: Span,
    pub value: StringIdentifier,
}

impl HasSpan for Keyword {
    fn span(&self) -> Span {
        self.span
    }
}
