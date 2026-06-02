use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::generics::GenericParameters;
use crate::cst::generics::SingleGenericParameter;
use crate::cst::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IntMaskType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameters: GenericParameters<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IntMaskOfType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameter: SingleGenericParameter<'arena>,
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
