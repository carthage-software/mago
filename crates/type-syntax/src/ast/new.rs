use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::SingleGenericParameter;
use crate::ast::keyword::Keyword;

/// The `new<X>` utility type.
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NewType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameter: SingleGenericParameter<'arena>,
}

impl HasSpan for NewType<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.parameter.span())
    }
}

impl std::fmt::Display for NewType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}<{}>", self.keyword, self.parameter)
    }
}
