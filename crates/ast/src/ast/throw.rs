use bumpalo::boxed::Box;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Throw<'a> {
    pub throw: Keyword,
    pub exception: Box<'a, Expression<'a>>,
}
impl HasSpan for Throw<'_> {
    fn span(&self) -> Span {
        self.throw.span().join(self.exception.span())
    }
}
