use ordered_float::OrderedFloat;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_interner::StringIdentifier;
use fennec_span::HasSpan;
use fennec_span::Span;

use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum LiteralExpression {
    String(LiteralString),
    Integer(LiteralInteger),
    Float(LiteralFloat),
    True(Keyword),
    False(Keyword),
    Null(Keyword),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum LiteralStringKind {
    SingleQuoted,
    DoubleQuoted,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct LiteralString {
    pub kind: LiteralStringKind,
    pub span: Span,
    pub value: StringIdentifier,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct LiteralInteger {
    pub span: Span,
    pub raw: StringIdentifier,
    pub value: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct LiteralFloat {
    pub span: Span,
    pub raw: StringIdentifier,
    pub value: OrderedFloat<f64>,
}

impl HasSpan for LiteralExpression {
    fn span(&self) -> Span {
        match self {
            LiteralExpression::String(value) => value.span(),
            LiteralExpression::Integer(value) => value.span(),
            LiteralExpression::Float(value) => value.span(),
            LiteralExpression::True(value) => value.span(),
            LiteralExpression::False(value) => value.span(),
            LiteralExpression::Null(value) => value.span(),
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
