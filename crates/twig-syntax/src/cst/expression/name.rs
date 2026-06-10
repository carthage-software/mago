use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Name<'arena> {
    pub name: &'arena [u8],
    pub span: Span,
}

impl HasSpan for Name<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
