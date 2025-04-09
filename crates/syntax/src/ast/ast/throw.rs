use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Throw {
    pub throw: Keyword,
    pub exception: Box<Expression>,
}

impl HasSpan for Throw {
    fn span(&self) -> Span {
        self.throw.span().join(self.exception.span())
    }
}
