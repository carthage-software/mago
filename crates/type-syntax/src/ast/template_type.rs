use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::keyword::Keyword;

/// The `template-type<Object, ClassName, TemplateName>` utility type.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TemplateTypeType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: GenericParameters<'input>,
}

impl HasSpan for TemplateTypeType<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.parameters.span())
    }
}

impl std::fmt::Display for TemplateTypeType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.keyword, self.parameters)
    }
}
