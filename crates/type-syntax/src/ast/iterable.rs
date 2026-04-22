use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IterableType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameters: Option<GenericParameters<'arena>>,
}

impl HasSpan for IterableType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl std::fmt::Display for IterableType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.parameters {
            write!(f, "{}{}", self.keyword, parameters)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}
