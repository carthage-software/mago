use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Flush<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub close_tag: Span,
}

impl HasSpan for Flush<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.close_tag)
    }
}
