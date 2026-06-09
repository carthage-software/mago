use std::fmt;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SyntaxError {
    /// A lexer-level unexpected character while inside a tag/expression.
    UnexpectedCharacter(FileId, u8, Position),
    /// A verbatim block was opened but never closed.
    UnclosedVerbatim(FileId, Position),
    /// A comment was opened but never closed.
    UnclosedComment(FileId, Position),
    /// A string literal was opened but never closed.
    UnclosedString(FileId, Position),
    /// An opening bracket `(`, `[`, or `{` was never matched by its closing counterpart.
    UnclosedBracket(FileId, u8, Position),
    /// A closing bracket was encountered with no matching opening bracket.
    UnmatchedBracket(FileId, u8, Position),
    /// Expression ran past the end of a tag without a closing marker.
    UnclosedTag(FileId, &'static str, Position),
}

impl SyntaxError {
    #[inline]
    #[must_use]
    pub const fn position(&self) -> Position {
        match self {
            SyntaxError::UnexpectedCharacter(_, _, p) => *p,
            SyntaxError::UnclosedVerbatim(_, p) => *p,
            SyntaxError::UnclosedComment(_, p) => *p,
            SyntaxError::UnclosedString(_, p) => *p,
            SyntaxError::UnclosedBracket(_, _, p) => *p,
            SyntaxError::UnmatchedBracket(_, _, p) => *p,
            SyntaxError::UnclosedTag(_, _, p) => *p,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParseError<'arena> {
    SyntaxError(SyntaxError),
    /// Generic "unexpected token" with a human-readable expectation string.
    UnexpectedToken(&'arena [u8], Span),
    /// Reached end-of-input while still expecting something.
    UnexpectedEof(FileId, &'arena [u8], Position),
    /// A tag name that the parser does not know.
    UnknownTag(&'arena [u8], Span),
    /// A closing tag (e.g. `endif`) whose kind does not match the open tag.
    MismatchedEndTag {
        expected: &'arena [u8],
        got: &'arena [u8],
        span: Span,
    },
    /// A generic syntax-level error produced by tag or expression parsers.
    Message(&'arena [u8], Span),
    /// Expression nesting exceeded the maximum supported recursion depth.
    /// Guards the recursive-descent parser against stack overflow on
    /// pathologically nested input.
    RecursionLimitExceeded(Span),
}

impl ParseError<'_> {
    #[must_use]
    pub fn message(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::UnexpectedCharacter(_, c, _) => {
                write!(f, "Unexpected character '{}'", *c as char)
            }
            SyntaxError::UnclosedVerbatim(_, _) => write!(f, "Unclosed verbatim block"),
            SyntaxError::UnclosedComment(_, _) => write!(f, "Unclosed comment"),
            SyntaxError::UnclosedString(_, _) => write!(f, "Unclosed string literal"),
            SyntaxError::UnclosedBracket(_, c, _) => write!(f, "Unclosed bracket '{}'", *c as char),
            SyntaxError::UnmatchedBracket(_, c, _) => write!(f, "Unmatched bracket '{}'", *c as char),
            SyntaxError::UnclosedTag(_, kind, _) => write!(f, "Unclosed {kind}"),
        }
    }
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::SyntaxError(err) => write!(f, "{err}"),
            ParseError::UnexpectedToken(msg, _) => {
                write!(f, "Unexpected token: {}", String::from_utf8_lossy(msg))
            }
            ParseError::UnexpectedEof(_, msg, _) => {
                write!(f, "Unexpected end of input: {}", String::from_utf8_lossy(msg))
            }
            ParseError::UnknownTag(name, _) => write!(f, "Unknown tag \"{}\"", String::from_utf8_lossy(name)),
            ParseError::MismatchedEndTag { expected, got, .. } => {
                write!(
                    f,
                    "Unexpected \"{}\" tag (expecting \"{}\")",
                    String::from_utf8_lossy(got),
                    String::from_utf8_lossy(expected)
                )
            }
            ParseError::Message(msg, _) => f.write_str(&String::from_utf8_lossy(msg)),
            ParseError::RecursionLimitExceeded(_) => f.write_str("Maximum recursion depth exceeded"),
        }
    }
}

impl std::error::Error for SyntaxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::error::Error for ParseError<'_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::SyntaxError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<SyntaxError> for ParseError<'_> {
    fn from(err: SyntaxError) -> Self {
        ParseError::SyntaxError(err)
    }
}

impl HasSpan for SyntaxError {
    fn span(&self) -> Span {
        let (file_id, position) = match self {
            SyntaxError::UnexpectedCharacter(file_id, _, p) => (*file_id, *p),
            SyntaxError::UnclosedVerbatim(file_id, p) => (*file_id, *p),
            SyntaxError::UnclosedComment(file_id, p) => (*file_id, *p),
            SyntaxError::UnclosedString(file_id, p) => (*file_id, *p),
            SyntaxError::UnclosedBracket(file_id, _, p) => (*file_id, *p),
            SyntaxError::UnmatchedBracket(file_id, _, p) => (*file_id, *p),
            SyntaxError::UnclosedTag(file_id, _, p) => (*file_id, *p),
        };

        Span::new(file_id, position, position)
    }
}

impl HasSpan for ParseError<'_> {
    fn span(&self) -> Span {
        match self {
            ParseError::SyntaxError(err) => err.span(),
            ParseError::UnexpectedToken(_, s) => *s,
            ParseError::UnexpectedEof(file_id, _, p) => Span::new(*file_id, *p, *p),
            ParseError::UnknownTag(_, s) => *s,
            ParseError::MismatchedEndTag { span, .. } => *span,
            ParseError::Message(_, s) => *s,
            ParseError::RecursionLimitExceeded(s) => *s,
        }
    }
}
