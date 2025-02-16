use bumpalo::boxed::Box;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::argument::ArgumentList;
use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Instantiation<'a> {
    pub new: Keyword,
    pub class: Box<'a, Expression<'a>>,
    pub arguments: Option<ArgumentList<'a>>,
}

impl HasSpan for Instantiation<'_> {
    fn span(&self) -> Span {
        if let Some(arguments) = &self.arguments {
            self.new.span().join(arguments.span())
        } else {
            self.new.span().join(self.class.span())
        }
    }
}
