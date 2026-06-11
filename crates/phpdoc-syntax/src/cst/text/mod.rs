use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::Tag;
use crate::cst::code::Code;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Text<'arena> {
    pub span: Span,
    pub segments: &'arena [TextSegment<'arena>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TextSegment<'arena> {
    PlainText(PlainText<'arena>),
    InlineCode(Code<'arena>),
    InlineTag(InlineTag<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InlineTag<'arena> {
    pub left_brace: Span,
    pub tag: &'arena Tag<'arena>,
    pub right_brace: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PlainText<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

impl HasSpan for Text<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for TextSegment<'_> {
    fn span(&self) -> Span {
        match self {
            TextSegment::InlineCode(code) => code.span(),
            TextSegment::PlainText(plain) => plain.span(),
            TextSegment::InlineTag(tag) => tag.span(),
        }
    }
}

impl HasSpan for InlineTag<'_> {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}

impl HasSpan for PlainText<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
