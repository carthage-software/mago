use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum OpeningTag<'arena> {
    Full(FullOpeningTag<'arena>),
    Short(ShortOpeningTag),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FullOpeningTag<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ShortOpeningTag {
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClosingTag {
    pub span: Span,
}

impl HasSpan for OpeningTag<'_> {
    fn span(&self) -> Span {
        match &self {
            OpeningTag::Full(t) => t.span(),
            OpeningTag::Short(t) => t.span(),
        }
    }
}

impl HasSpan for FullOpeningTag<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ShortOpeningTag {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ClosingTag {
    fn span(&self) -> Span {
        self.span
    }
}
