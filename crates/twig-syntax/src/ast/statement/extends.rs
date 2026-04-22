use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Extends<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub template: Expression<'arena>,
    pub close_tag: Span,
}

impl HasSpan for Extends<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.close_tag)
    }
}
