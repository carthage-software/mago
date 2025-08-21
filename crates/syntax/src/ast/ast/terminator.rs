use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::tag::ClosingTag;
use crate::ast::ast::tag::OpeningTag;

/// A statement terminator.
///
/// A PHP statement can be terminated with a semicolon `;` or a closing tag `?>`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Terminator<'arena> {
    /// A semicolon.
    Semicolon(Span),
    /// A closing tag.
    ClosingTag(ClosingTag),
    /// A closing tag followed immediately by an opening tag.
    TagPair(ClosingTag, OpeningTag<'arena>),
}

impl Terminator<'_> {
    #[must_use]
    #[inline]
    pub const fn is_semicolon(&self) -> bool {
        matches!(self, Terminator::Semicolon(_))
    }

    #[must_use]
    #[inline]
    pub const fn is_closing_tag(&self) -> bool {
        matches!(self, Terminator::ClosingTag(_))
    }
}

impl HasSpan for Terminator<'_> {
    fn span(&self) -> Span {
        match self {
            Terminator::Semicolon(s) => *s,
            Terminator::ClosingTag(t) => t.span(),
            Terminator::TagPair(c, o) => c.span().join(o.span()),
        }
    }
}
