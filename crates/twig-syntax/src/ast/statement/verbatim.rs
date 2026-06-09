use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Verbatim<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub close_tag: Span,
    pub body: &'arena [u8],
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_close_tag: Span,
}

impl HasSpan for Verbatim<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.end_close_tag)
    }
}
