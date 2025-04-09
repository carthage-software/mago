use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum OpeningTag {
    Full(FullOpeningTag),
    Short(ShortOpeningTag),
    Echo(EchoOpeningTag),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct FullOpeningTag {
    pub span: Span,
    pub value: StringIdentifier,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ShortOpeningTag {
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct EchoOpeningTag {
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ClosingTag {
    pub span: Span,
}

impl HasSpan for OpeningTag {
    fn span(&self) -> Span {
        match &self {
            OpeningTag::Full(t) => t.span(),
            OpeningTag::Short(t) => t.span(),
            OpeningTag::Echo(t) => t.span(),
        }
    }
}

impl HasSpan for FullOpeningTag {
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
