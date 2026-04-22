use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ArgumentList;
use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Sequence;
use crate::ast::TokenSeparatedSequence;
use crate::ast::statement::Statement;

/// One filter application in an `{% apply %}` pipeline (`name(args?)`
/// between `|` separators).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct FilterApplication<'arena> {
    pub name: Identifier<'arena>,
    pub argument_list: Option<ArgumentList<'arena>>,
}

impl HasSpan for FilterApplication<'_> {
    fn span(&self) -> Span {
        let end = self.argument_list.as_ref().map(HasSpan::span).unwrap_or(self.name.span);
        self.name.span.join(end)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Apply<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub filters: TokenSeparatedSequence<'arena, FilterApplication<'arena>>,
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_close_tag: Span,
}

impl HasSpan for Apply<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.end_close_tag)
    }
}
