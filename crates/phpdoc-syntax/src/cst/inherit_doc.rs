use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct InheritDoc {
    pub span: Span,
}

impl HasSpan for InheritDoc {
    fn span(&self) -> Span {
        self.span
    }
}
