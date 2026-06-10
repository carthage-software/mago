use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::ArgumentList;
use crate::cst::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Call<'arena> {
    pub callee: &'arena Expression<'arena>,
    pub argument_list: ArgumentList<'arena>,
}

impl HasSpan for Call<'_> {
    fn span(&self) -> Span {
        self.callee.span().join(self.argument_list.right_parenthesis)
    }
}
