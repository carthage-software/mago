#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_phpdoc_syntax::error::ParseError;
use mago_span::HasSpan;
use mago_span::Span;

use mago_allocator::copy::CopyInto;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct AnnotationError {
    pub span: Span,
    pub kind: AnnotationErrorKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AnnotationErrorKind {
    UnexpectedToken,
    UnexpectedEndOfInput,
    UnclosedInlineTag,
    UnclosedInlineCode,
    UnclosedCodeBlock,
    MalformedCodeBlock,
    UnclosedLiteralString,
    RecursionLimitExceeded,
}

impl CopyInto for AnnotationError {
    type Output<'arena> = AnnotationError;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        AnnotationError { span: self.span, kind: self.kind }
    }
}

impl CopyInto for AnnotationErrorKind {
    type Output<'arena> = AnnotationErrorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl From<ParseError> for AnnotationError {
    fn from(error: ParseError) -> Self {
        let kind = match error {
            ParseError::UnexpectedToken(_) => AnnotationErrorKind::UnexpectedToken,
            ParseError::UnexpectedEndOfInput(_) => AnnotationErrorKind::UnexpectedEndOfInput,
            ParseError::UnclosedInlineTag(_) => AnnotationErrorKind::UnclosedInlineTag,
            ParseError::UnclosedInlineCode(_) => AnnotationErrorKind::UnclosedInlineCode,
            ParseError::UnclosedCodeBlock(_) => AnnotationErrorKind::UnclosedCodeBlock,
            ParseError::MalformedCodeBlock(_) => AnnotationErrorKind::MalformedCodeBlock,
            ParseError::UnclosedLiteralString(_) => AnnotationErrorKind::UnclosedLiteralString,
            ParseError::RecursionLimitExceeded(_) => AnnotationErrorKind::RecursionLimitExceeded,
        };

        AnnotationError { span: error.span(), kind }
    }
}

impl HasSpan for AnnotationError {
    fn span(&self) -> Span {
        self.span
    }
}
