use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::LiteralIntOrFloatType;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NegatedType<'arena> {
    pub minus: Span,
    pub number: LiteralIntOrFloatType<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PositedType<'arena> {
    pub plus: Span,
    pub number: LiteralIntOrFloatType<'arena>,
}

impl HasSpan for NegatedType<'_> {
    fn span(&self) -> Span {
        self.minus.join(self.number.span())
    }
}

impl HasSpan for PositedType<'_> {
    fn span(&self) -> Span {
        self.plus.join(self.number.span())
    }
}

impl std::fmt::Display for NegatedType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "-{}", self.number)
    }
}

impl std::fmt::Display for PositedType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "+{}", self.number)
    }
}
