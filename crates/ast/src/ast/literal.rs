use ordered_float::OrderedFloat;
use serde::Serialize;
use strum::Display;

use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::keyword::Keyword;

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Literal {
    String(LiteralString),
    Integer(LiteralInteger),
    Float(LiteralFloat),
    True(Keyword),
    False(Keyword),
    Null(Keyword),
}

#[derive(Debug, Hash, Serialize, Display, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
#[repr(C)]
pub enum LiteralStringKind {
    SingleQuoted,
    DoubleQuoted,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct LiteralString {
    pub kind: LiteralStringKind,
    pub span: Span,
    pub value: StringIdentifier,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct LiteralInteger {
    pub span: Span,
    pub raw: StringIdentifier,
    pub value: Option<u64>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct LiteralFloat {
    pub span: Span,
    pub raw: StringIdentifier,
    pub value: OrderedFloat<f64>,
}

impl HasSpan for Literal {
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

impl HasSpan for LiteralString {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for LiteralInteger {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for LiteralFloat {
    fn span(&self) -> Span {
        self.span
    }
}
