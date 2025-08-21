use ordered_float::OrderedFloat;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Literal<'arena> {
    String(LiteralString<'arena>),
    Integer(LiteralInteger<'arena>),
    Float(LiteralFloat<'arena>),
    True(Keyword<'arena>),
    False(Keyword<'arena>),
    Null(Keyword<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum LiteralStringKind {
    SingleQuoted,
    DoubleQuoted,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralString<'arena> {
    pub kind: Option<LiteralStringKind>,
    pub span: Span,
    pub raw: &'arena str,
    pub value: Option<&'arena str>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralInteger<'arena> {
    pub span: Span,
    pub raw: &'arena str,
    pub value: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralFloat<'arena> {
    pub span: Span,
    pub raw: &'arena str,
    pub value: OrderedFloat<f64>,
}

impl HasSpan for Literal<'_> {
    fn span(&self) -> Span {
        match self {
            Literal::String(value) => value.span(),
            Literal::Integer(value) => value.span(),
            Literal::Float(value) => value.span(),
            Literal::True(value) => value.span(),
            Literal::False(value) => value.span(),
            Literal::Null(value) => value.span(),
        }
    }
}

impl HasSpan for LiteralString<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for LiteralInteger<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for LiteralFloat<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
