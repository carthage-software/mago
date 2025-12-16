use ordered_float::OrderedFloat;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralIntType<'input> {
    pub span: Span,
    pub value: u64,
    pub raw: &'input str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralFloatType<'input> {
    pub span: Span,
    pub value: OrderedFloat<f64>,
    pub raw: &'input str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum LiteralIntOrFloatType<'input> {
    Int(LiteralIntType<'input>),
    Float(LiteralFloatType<'input>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralStringType<'input> {
    pub span: Span,
    pub value: &'input str, // unquoted
    pub raw: &'input str,
}

impl HasSpan for LiteralFloatType<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for LiteralIntType<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for LiteralIntOrFloatType<'_> {
    fn span(&self) -> Span {
        match self {
            LiteralIntOrFloatType::Int(int) => int.span(),
            LiteralIntOrFloatType::Float(float) => float.span(),
        }
    }
}

impl HasSpan for LiteralStringType<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl std::fmt::Display for LiteralIntType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl std::fmt::Display for LiteralFloatType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl std::fmt::Display for LiteralIntOrFloatType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralIntOrFloatType::Int(int) => write!(f, "{int}"),
            LiteralIntOrFloatType::Float(float) => write!(f, "{float}"),
        }
    }
}

impl std::fmt::Display for LiteralStringType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}
