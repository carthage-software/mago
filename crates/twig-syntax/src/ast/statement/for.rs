use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Sequence;
use crate::ast::TokenSeparatedSequence;
use crate::ast::expression::Expression;
use crate::ast::statement::Statement;
use crate::ast::statement::r#if::ElseBranch;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ForIfClause<'arena> {
    pub keyword: Keyword<'arena>,
    pub condition: Expression<'arena>,
}

impl HasSpan for ForIfClause<'_> {
    fn span(&self) -> Span {
        self.keyword.span.join(self.condition.span())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct For<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub targets: TokenSeparatedSequence<'arena, Identifier<'arena>>,
    pub in_keyword: Keyword<'arena>,
    pub sequence: Expression<'arena>,
    pub if_clause: Option<ForIfClause<'arena>>,
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
    pub else_branch: Option<ElseBranch<'arena>>,
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_close_tag: Span,
}

impl HasSpan for For<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.end_close_tag)
    }
}
