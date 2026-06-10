use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::identifier::Identifier;
use crate::cst::keyword::Keyword;
use crate::cst::r#type::Type;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct WhereTagValue<'arena> {
    pub name: Identifier<'arena>,
    pub modifier: WhereTagValueModifier<'arena>,
    pub r#type: &'arena Type<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
pub enum WhereTagValueModifier<'arena> {
    Is(Keyword<'arena>),
    Colon(Span),
}

impl HasSpan for WhereTagValue<'_> {
    fn span(&self) -> Span {
        self.name.span().join(self.r#type.span())
    }
}

impl HasSpan for WhereTagValueModifier<'_> {
    fn span(&self) -> Span {
        match self {
            WhereTagValueModifier::Is(keyword) => keyword.span(),
            WhereTagValueModifier::Colon(span) => *span,
        }
    }
}
