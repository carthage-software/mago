use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct IterableType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

impl HasSpan for IterableType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}
