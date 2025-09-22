use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::GenericParameters;
use crate::ast::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ObjectType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

impl HasSpan for ObjectType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl std::fmt::Display for ObjectType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.parameters {
            write!(f, "{}{}", self.keyword, parameters)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}
