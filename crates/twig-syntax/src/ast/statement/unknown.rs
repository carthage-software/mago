use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Unknown<'arena> {
    pub open_tag: Span,
    pub name: Identifier<'arena>,
    pub raw: &'arena [u8],
    pub close_tag: Span,
}

impl HasSpan for Unknown<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.close_tag)
    }
}
