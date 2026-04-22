use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Sequence;
use crate::ast::TokenSeparatedSequence;
use crate::ast::expression::Expression;
use crate::ast::statement::Statement;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MacroArgument<'arena> {
    pub name: Identifier<'arena>,
    pub equal: Option<Span>,
    pub default: Option<Expression<'arena>>,
}

impl HasSpan for MacroArgument<'_> {
    fn span(&self) -> Span {
        let end = self.default.as_ref().map(HasSpan::span).unwrap_or(self.name.span);
        self.name.span.join(end)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Macro<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub name: Identifier<'arena>,
    pub left_parenthesis: Span,
    pub arguments: TokenSeparatedSequence<'arena, MacroArgument<'arena>>,
    pub right_parenthesis: Span,
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_name: Option<Identifier<'arena>>,
    pub end_close_tag: Span,
}

impl HasSpan for Macro<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.end_close_tag)
    }
}
