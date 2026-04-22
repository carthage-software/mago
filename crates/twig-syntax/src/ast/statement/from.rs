use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::TokenSeparatedSequence;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ImportedMacro<'arena> {
    pub from: Identifier<'arena>,
    pub as_keyword: Option<Keyword<'arena>>,
    pub to: Option<Identifier<'arena>>,
}

impl HasSpan for ImportedMacro<'_> {
    fn span(&self) -> Span {
        let end = self.to.map(|i| i.span).unwrap_or(self.from.span);
        self.from.span.join(end)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct From<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub template: Expression<'arena>,
    pub import_keyword: Keyword<'arena>,
    pub names: TokenSeparatedSequence<'arena, ImportedMacro<'arena>>,
    pub close_tag: Span,
}

impl HasSpan for From<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.close_tag)
    }
}
