use bumpalo::boxed::Box;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Conditional<'a> {
    pub condition: Box<'a, Expression<'a>>,
    pub question_mark: Span,
    pub then: Option<Box<'a, Expression<'a>>>,
    pub colon: Span,
    pub r#else: Box<'a, Expression<'a>>,
}

impl HasSpan for Conditional<'_> {
    fn span(&self) -> Span {
        self.condition.span().join(self.r#else.span())
    }
}
