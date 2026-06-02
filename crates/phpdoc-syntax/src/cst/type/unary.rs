use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::r#type::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NegatedType<'arena> {
    pub minus: Span,
    pub operand: &'arena Type<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PositedType<'arena> {
    pub plus: Span,
    pub operand: &'arena Type<'arena>,
}

impl HasSpan for NegatedType<'_> {
    fn span(&self) -> Span {
        self.minus.join(self.operand.span())
    }
}

impl HasSpan for PositedType<'_> {
    fn span(&self) -> Span {
        self.plus.join(self.operand.span())
    }
}

impl std::fmt::Display for NegatedType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "-{}", self.operand)
    }
}

impl std::fmt::Display for PositedType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "+{}", self.operand)
    }
}
