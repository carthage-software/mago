use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::Keyword;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Code<'arena> {
    pub span: Span,
    pub bound: u8,
    pub left_bound: Span,
    pub language: Option<Keyword<'arena>>,
    pub value: &'arena [u8],
    pub right_bound: Span,
}

impl HasSpan for Code<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
