use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Name<'arena> {
    pub name: &'arena str,
    pub span: Span,
}

impl HasSpan for Name<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
