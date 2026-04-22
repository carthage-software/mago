use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ArgumentList;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Call<'arena> {
    pub callee: &'arena Expression<'arena>,
    pub argument_list: ArgumentList<'arena>,
}

impl HasSpan for Call<'_> {
    fn span(&self) -> Span {
        self.callee.span().join(self.argument_list.right_parenthesis)
    }
}
