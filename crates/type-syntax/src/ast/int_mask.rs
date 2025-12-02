use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::generics::SingleGenericParameter;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IntMaskType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: GenericParameters<'input>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IntMaskOfType<'input> {
    pub keyword: Keyword<'input>,
    pub parameter: SingleGenericParameter<'input>,
}

impl HasSpan for IntMaskType<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.parameters.span())
    }
}

impl HasSpan for IntMaskOfType<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.parameter.span())
    }
}

impl std::fmt::Display for IntMaskType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.keyword, self.parameters)
    }
}

impl std::fmt::Display for IntMaskOfType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.keyword, self.parameter)
    }
}
