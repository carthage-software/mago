use ordered_float::OrderedFloat;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Literal<'arena> {
    pub span: Span,
    pub kind: LiteralKind<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum LiteralKind<'arena> {
    String(LiteralString<'arena>),
    Integer(LiteralInteger<'arena>),
    Float(LiteralFloat<'arena>),
    True,
    False,
    Null,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum LiteralStringKind {
    SingleQuoted,
    DoubleQuoted,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralString<'arena> {
    pub kind: LiteralStringKind,
    pub raw: &'arena [u8],
    pub value: Option<&'arena [u8]>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralInteger<'arena> {
    pub raw: &'arena [u8],
    pub value: Option<u64>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralFloat<'arena> {
    pub raw: &'arena [u8],
    pub value: OrderedFloat<f64>,
}

impl HasSpan for Literal<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
