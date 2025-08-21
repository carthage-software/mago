use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Pipe<'arena> {
    /// The expression whose value is passed as the first argument.
    pub input: &'arena Expression<'arena>,
    /// The span of the pipe operator `|>`.
    pub operator: Span,
    /// The expression that must resolve to a callable.
    pub callable: &'arena Expression<'arena>,
}

impl HasSpan for Pipe<'_> {
    fn span(&self) -> Span {
        self.input.span().join(self.callable.span())
    }
}
