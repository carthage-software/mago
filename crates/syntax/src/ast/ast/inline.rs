use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum InlineKind {
    Text,
    Shebang,
}

/// Represents inline text within a PHP script.
///
/// # Example:
///
/// ```php
/// This is an inline text.
/// <?php
///   // PHP code
/// ?>
/// This is another inline text.
/// ```
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Inline<'arena> {
    pub kind: InlineKind,
    pub span: Span,
    pub value: &'arena str,
}

impl HasSpan for Inline<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
