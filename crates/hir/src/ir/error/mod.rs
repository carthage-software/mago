pub mod annotation;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum ErrorKind {
    UnrecognizedToken,
    UnexpectedEndOfFile,
    UnexpectedToken,
    UnclosedLiteralString,
    RecursionLimitExceeded,
}

impl HasSpan for Error {
    fn span(&self) -> Span {
        self.span
    }
}
