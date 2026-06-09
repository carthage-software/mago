use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Keyword<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

impl HasSpan for Keyword<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
