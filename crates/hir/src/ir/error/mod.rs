pub mod annotation;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use mago_allocator::copy::CopyInto;

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

impl CopyInto for Error {
    type Output<'arena> = Error;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Error { span: self.span, kind: self.kind }
    }
}

impl CopyInto for ErrorKind {
    type Output<'arena> = ErrorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl HasSpan for Error {
    fn span(&self) -> Span {
        self.span
    }
}
