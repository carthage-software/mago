use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ArgumentList;
use crate::ast::Identifier;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Filter<'arena> {
    pub operand: &'arena Expression<'arena>,
    pub pipe: Span,
    pub name: Identifier<'arena>,
    pub argument_list: Option<ArgumentList<'arena>>,
}

impl HasSpan for Filter<'_> {
    fn span(&self) -> Span {
        let end = self.argument_list.as_ref().map(HasSpan::span).unwrap_or(self.name.span);
        self.operand.span().join(end)
    }
}
