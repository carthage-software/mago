use bumpalo::boxed::Box;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Clone<'a> {
    pub clone: Keyword,
    pub object: Box<'a, Expression<'a>>,
}

impl HasSpan for Clone<'_> {
    fn span(&self) -> Span {
        self.clone.span().join(self.object.span())
    }
}
