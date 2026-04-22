use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;
use crate::ast::Sequence;
use crate::ast::expression::Expression;
use crate::ast::statement::Statement;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct CacheOption<'arena> {
    pub keyword: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub value: Expression<'arena>,
    pub right_parenthesis: Span,
}

impl HasSpan for CacheOption<'_> {
    fn span(&self) -> Span {
        self.keyword.span.join(self.right_parenthesis)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Cache<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub key: Expression<'arena>,
    pub ttl: Option<CacheOption<'arena>>,
    pub tags: Option<CacheOption<'arena>>,
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_close_tag: Span,
}

impl HasSpan for Cache<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.end_close_tag)
    }
}
