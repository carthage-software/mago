use mago_span::HasSpan;
use mago_span::Span;

/// Raw template text (including whitespace between tags).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Text<'arena> {
    pub value: &'arena [u8],
    pub span: Span,
}

impl HasSpan for Text<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
