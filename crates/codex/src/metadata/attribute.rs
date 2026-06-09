use mago_span::HasSpan;
use mago_span::Span;
use mago_word::Word;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct AttributeMetadata {
    pub name: Word,
    pub span: Span,
}

impl HasSpan for AttributeMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
