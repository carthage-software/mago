use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Name<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

impl HasSpan for Name<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
