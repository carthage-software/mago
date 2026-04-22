use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IgnoreMissingClause<'arena> {
    pub ignore_keyword: Keyword<'arena>,
    pub missing_keyword: Keyword<'arena>,
}

impl HasSpan for IgnoreMissingClause<'_> {
    fn span(&self) -> Span {
        self.ignore_keyword.span.join(self.missing_keyword.span)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct WithExpressionClause<'arena> {
    pub with_keyword: Keyword<'arena>,
    pub variables: Expression<'arena>,
}

impl HasSpan for WithExpressionClause<'_> {
    fn span(&self) -> Span {
        self.with_keyword.span.join(self.variables.span())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Include<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub template: Expression<'arena>,
    pub ignore_missing: Option<IgnoreMissingClause<'arena>>,
    pub with_clause: Option<WithExpressionClause<'arena>>,
    pub only_keyword: Option<Keyword<'arena>>,
    pub close_tag: Span,
}

impl HasSpan for Include<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.close_tag)
    }
}
