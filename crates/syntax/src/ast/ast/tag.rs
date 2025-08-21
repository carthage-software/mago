use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum OpeningTag<'arena> {
    Full(FullOpeningTag<'arena>),
    Short(ShortOpeningTag),
    Echo(EchoOpeningTag),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct FullOpeningTag<'arena> {
    pub span: Span,
    pub value: &'arena str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ShortOpeningTag {
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EchoOpeningTag {
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClosingTag {
    pub span: Span,
}

impl HasSpan for OpeningTag<'_> {
    fn span(&self) -> Span {
        match &self {
            OpeningTag::Full(t) => t.span(),
            OpeningTag::Short(t) => t.span(),
            OpeningTag::Echo(t) => t.span(),
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

impl HasSpan for EchoOpeningTag {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ClosingTag {
    fn span(&self) -> Span {
        self.span
    }
}
