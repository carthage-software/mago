use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::argument::ArgumentList;
use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Instantiation {
    pub new: Keyword,
    pub class: Box<Expression>,
    pub arguments: Option<ArgumentList>,
}

impl HasSpan for Instantiation {
    fn span(&self) -> Span {
        if let Some(arguments) = &self.arguments {
            self.new.span().join(arguments.span())
        } else {
            self.new.span().join(self.class.span())
        }
    }
}
