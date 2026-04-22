use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax_core::ast::Sequence;

use crate::ast::Type;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericParameterEntry<'arena> {
    pub inner: Type<'arena>,
    pub comma: Option<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericParameters<'arena> {
    pub less_than: Span,
    pub entries: Sequence<'arena, GenericParameterEntry<'arena>>,
    pub greater_than: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SingleGenericParameter<'arena> {
    pub less_than: Span,
    pub entry: &'arena GenericParameterEntry<'arena>,
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

impl HasSpan for SingleGenericParameter<'_> {
    fn span(&self) -> Span {
        self.less_than.join(self.greater_than)
    }
}

impl std::fmt::Display for GenericParameterEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::fmt::Display for SingleGenericParameter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.entry)
    }
}

impl std::fmt::Display for GenericParameters<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<")?;
        for (i, entry) in self.entries.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{entry}")?;
        }
        write!(f, ">")
    }
}
