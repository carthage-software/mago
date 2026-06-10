use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::Keyword;
use crate::cst::Sequence;
use crate::cst::expression::Expression;
use crate::cst::statement::Statement;
use crate::cst::statement::include::IgnoreMissingClause;
use crate::cst::statement::include::WithExpressionClause;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Embed<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub template: Expression<'arena>,
    pub ignore_missing: Option<IgnoreMissingClause<'arena>>,
    pub with_clause: Option<WithExpressionClause<'arena>>,
    pub only_keyword: Option<Keyword<'arena>>,
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_close_tag: Span,
}

impl HasSpan for Embed<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.end_close_tag)
    }
}
