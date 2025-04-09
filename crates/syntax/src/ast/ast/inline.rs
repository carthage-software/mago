use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C)]
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Inline {
    pub kind: InlineKind,
    pub span: Span,
    pub value: StringIdentifier,
}

impl HasSpan for Inline {
    fn span(&self) -> Span {
        self.span
    }
}
