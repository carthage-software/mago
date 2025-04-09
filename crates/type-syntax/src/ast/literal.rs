use ordered_float::OrderedFloat;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct LiteralIntType<'input> {
    pub span: Span,
    pub value: u64,
    pub raw: &'input str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct LiteralFloatType<'input> {
    pub span: Span,
    pub value: OrderedFloat<f64>,
    pub raw: &'input str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
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

impl HasSpan for LiteralStringType<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
