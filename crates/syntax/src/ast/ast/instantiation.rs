use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::argument::ArgumentList;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Instantiation<'arena> {
    pub new: Keyword<'arena>,
    pub class: &'arena Expression<'arena>,
    pub argument_list: Option<ArgumentList<'arena>>,
}

impl HasSpan for Instantiation<'_> {
    fn span(&self) -> Span {
        if let Some(argument_list) = &self.argument_list {
            self.new.span().join(argument_list.span())
        } else {
            self.new.span().join(self.class.span())
        }
    }
}
