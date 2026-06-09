use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::SingleGenericParameter;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ValueOfType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameter: SingleGenericParameter<'arena>,
}

impl HasSpan for ValueOfType<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.parameter.span())
    }
}

impl std::fmt::Display for ValueOfType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}<{}>", self.keyword, self.parameter)
    }
}
