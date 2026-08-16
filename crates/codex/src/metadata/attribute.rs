use mago_span::HasSpan;
use mago_span::Span;
use mago_word::Word;

use crate::ttype::union::TUnion;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct AttributeArgumentMetadata {
    pub name: Option<Word>,
    pub span: Span,
    pub name_span: Option<Span>,
    pub value_span: Option<Span>,
    pub value_type: Option<TUnion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct AttributeMetadata {
    pub name: Word,
    pub span: Span,
    pub arguments: Vec<AttributeArgumentMetadata>,
}

impl HasSpan for AttributeMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
