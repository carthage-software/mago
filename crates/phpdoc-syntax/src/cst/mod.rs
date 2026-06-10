use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax_core::cst::Sequence;

use crate::error::ParseError;

pub use crate::cst::element::*;
pub use crate::cst::expression::*;
pub use crate::cst::identifier::*;
pub use crate::cst::keyword::*;
pub use crate::cst::tag::*;
pub use crate::cst::text::*;
pub use crate::cst::trivia::*;
pub use crate::cst::variable::*;

pub mod code;
pub mod element;
pub mod expression;
pub mod identifier;
pub mod keyword;
pub mod tag;
pub mod text;
pub mod trivia;
pub mod r#type;
pub mod variable;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Document<'arena> {
    pub span: Span,
    pub trivia: Sequence<'arena, Trivia<'arena>>,
    pub elements: Sequence<'arena, Element<'arena>>,
    pub errors: &'arena [ParseError],
}

impl Document<'_> {
    #[inline]
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn has_inherit_doc(&self) -> bool {
        self.elements.iter().any(|element| match element {
            Element::Tag(tag) => tag.name.value.eq_ignore_ascii_case(b"inheritDoc"),
            Element::Text(text) => text.segments.iter().any(|segment| match segment {
                TextSegment::InlineTag(inline_tag) => inline_tag.tag.name.value.eq_ignore_ascii_case(b"inheritDoc"),
                _ => false,
            }),
            _ => false,
        })
    }
}

impl HasSpan for Document<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
