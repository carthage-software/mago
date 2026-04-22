use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

/// A keyword (tag name, `in`, `as`, `with`, `only`, `if`/`elseif`/`else`,
/// `endX`, etc.) paired with its span.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Keyword<'arena> {
    pub span: Span,
    pub value: &'arena str,
}

/// A plain identifier (loop target, macro argument name, block name,
/// macro name, alias) paired with its span.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Identifier<'arena> {
    pub span: Span,
    pub value: &'arena str,
}

impl HasSpan for Keyword<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Identifier<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
