use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::code::Code;
use crate::cst::tag::Tag;
use crate::cst::text::Text;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
pub enum Element<'arena> {
    Tag(&'arena Tag<'arena>),
    Text(&'arena Text<'arena>),
    Code(&'arena Code<'arena>),
}

impl HasSpan for Element<'_> {
    fn span(&self) -> Span {
        match self {
            Element::Tag(tag) => tag.span(),
            Element::Text(text) => text.span(),
            Element::Code(code) => code.span(),
        }
    }
}
