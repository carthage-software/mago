use ordered_float::OrderedFloat;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IntegerConstant<'arena> {
    pub span: Span,
    pub value: u64,
    pub raw: &'arena [u8],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FloatConstant<'arena> {
    pub span: Span,
    pub value: OrderedFloat<f64>,
    pub raw: &'arena [u8],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StringConstant<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
    pub raw: &'arena [u8],
}

impl HasSpan for IntegerConstant<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for FloatConstant<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for StringConstant<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
