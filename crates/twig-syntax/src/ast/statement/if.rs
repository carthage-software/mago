use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;
use crate::ast::Sequence;
use crate::ast::expression::Expression;
use crate::ast::statement::Statement;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IfBranch<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub condition: Expression<'arena>,
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ElseBranch<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct If<'arena> {
    pub branches: Sequence<'arena, IfBranch<'arena>>,
    pub else_branch: Option<ElseBranch<'arena>>,
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_close_tag: Span,
}

impl HasSpan for IfBranch<'_> {
    fn span(&self) -> Span {
        let end = self.body.last().map(HasSpan::span).unwrap_or(self.close_tag);
        self.open_tag.join(end)
    }
}

impl HasSpan for ElseBranch<'_> {
    fn span(&self) -> Span {
        let end = self.body.last().map(HasSpan::span).unwrap_or(self.close_tag);
        self.open_tag.join(end)
    }
}

impl HasSpan for If<'_> {
    fn span(&self) -> Span {
        let start = self.branches.first().map(HasSpan::span).unwrap_or(self.end_open_tag);
        start.join(self.end_close_tag)
    }
}
