use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Clone {
    pub clone: Keyword,
    pub object: Box<Expression>,
}

impl HasSpan for Clone {
    fn span(&self) -> Span {
        self.clone.span().join(self.object.span())
    }
}
