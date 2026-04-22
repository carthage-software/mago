use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::TokenSeparatedSequence;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ArrowFunction<'arena> {
    pub left_parenthesis: Option<Span>,
    pub parameters: TokenSeparatedSequence<'arena, Identifier<'arena>>,
    pub right_parenthesis: Option<Span>,
    pub fat_arrow: Span,
    pub body: &'arena Expression<'arena>,
}

impl HasSpan for ArrowFunction<'_> {
    fn span(&self) -> Span {
        let start =
            self.left_parenthesis.or_else(|| self.parameters.nodes.first().map(|i| i.span)).unwrap_or(self.fat_arrow);
        start.join(self.body.span())
    }
}
