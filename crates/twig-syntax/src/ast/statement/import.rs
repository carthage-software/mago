use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Import<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub template: Expression<'arena>,
    pub as_keyword: Keyword<'arena>,
    pub alias: Identifier<'arena>,
    pub close_tag: Span,
}

impl HasSpan for Import<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.close_tag)
    }
}
