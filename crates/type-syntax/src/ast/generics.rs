use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct GenericParameterEntry<'input> {
    pub inner: Type<'input>,
    pub comma: Option<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct GenericParameters<'input> {
    pub less_than: Span,
    pub entries: Vec<GenericParameterEntry<'input>>,
    pub greater_than: Span,
}

impl HasSpan for GenericParameterEntry<'_> {
    fn span(&self) -> Span {
        match &self.comma {
            Some(comma) => self.inner.span().join(*comma),
            None => self.inner.span(),
        }
    }
}

impl HasSpan for GenericParameters<'_> {
    fn span(&self) -> Span {
        self.less_than.join(self.greater_than)
    }
}
