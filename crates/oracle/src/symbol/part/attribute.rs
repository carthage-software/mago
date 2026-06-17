#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::path::Path;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct AppliedAttribute<'arena> {
    pub span: Span,
    pub name: Path<'arena>,
}

impl HasSpan for AppliedAttribute<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
