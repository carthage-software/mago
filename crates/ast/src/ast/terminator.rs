use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::tag::ClosingTag;
use crate::ast::tag::OpeningTag;

/// A statement terminator.
///
/// A PHP statement can be terminated with a semicolon `;` or a closing tag `?>`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Terminator {
    /// A semicolon.
    Semicolon(Span),
    /// A closing tag.
    ClosingTag(ClosingTag),
    /// A closing tag followed immediately by an opening tag.
    TagPair(ClosingTag, OpeningTag),
}

impl HasSpan for Terminator {
    fn span(&self) -> Span {
        match self {
            Terminator::Semicolon(s) => *s,
            Terminator::ClosingTag(t) => t.span(),
            Terminator::TagPair(c, o) => c.span().join(o.span()),
        }
    }
}
