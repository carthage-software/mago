use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;
use fennec_span::HasSpan;
use fennec_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Keyword {
    pub span: Span,
    pub value: StringIdentifier,
}

impl HasSpan for Keyword {
    fn span(&self) -> Span {
        self.span
    }
}
