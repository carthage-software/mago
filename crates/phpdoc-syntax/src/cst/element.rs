use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::tag::Tag;
use crate::cst::text::Text;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum Element<'arena> {
    Tag(&'arena Tag<'arena>),
    Text(Text<'arena>),
}

impl HasSpan for Element<'_> {
    fn span(&self) -> Span {
        match self {
            Element::Tag(tag) => tag.span(),
            Element::Text(text) => text.span(),
        }
    }
}
