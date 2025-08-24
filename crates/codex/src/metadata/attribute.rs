use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttributeMetadata {
    pub name: Atom,
    pub span: Span,
}

impl HasSpan for AttributeMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
