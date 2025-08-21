use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Clone<'arena> {
    pub clone: Keyword<'arena>,
    pub object: &'arena Expression<'arena>,
}

impl HasSpan for Clone<'_> {
    fn span(&self) -> Span {
        self.clone.span().join(self.object.span())
    }
}
