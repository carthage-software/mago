use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

use crate::ast::LiteralStringKind;
use crate::token::TokenKind;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum SyntaxError {
    UnexpectedToken(u8, Position),
    UnrecognizedToken(u8, Position),
    UnexpectedEndOfFile(Position),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ParseError {
    SyntaxError(SyntaxError),
    UnexpectedEndOfFile(Vec<TokenKind>, Position),
    UnexpectedToken(Vec<TokenKind>, TokenKind, Span),
    UnclosedLiteralString(LiteralStringKind, Span),
}

impl HasSpan for ParseError {
    fn span(&self) -> Span {
        match &self {
            ParseError::SyntaxError(syntax_error) => syntax_error.span(),
            ParseError::UnexpectedEndOfFile(_, position) => Span::new(*position, *position),
            ParseError::UnexpectedToken(_, _, span) => *span,
            ParseError::UnclosedLiteralString(_, span) => *span,
        }
    }
}

impl HasSpan for SyntaxError {
    fn span(&self) -> Span {
        let position = match self {
            Self::UnexpectedToken(_, p) => *p,
            Self::UnrecognizedToken(_, p) => *p,
            Self::UnexpectedEndOfFile(p) => *p,
        };

        Span::new(position, Position { offset: position.offset + 1, ..position })
    }
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UnexpectedToken(token, _) => &format!("Unexpected token `{}` (0x{:02X})", *token as char, token),
            Self::UnrecognizedToken(token, _) => &format!("Unrecognised token `{}` (0x{:02X})", *token as char, token),
            Self::UnexpectedEndOfFile(_) => "Unexpected end of file",
        };

        write!(f, "{}", message)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ParseError::SyntaxError(e) => {
                return write!(f, "{}", e);
            }
            ParseError::UnexpectedEndOfFile(expected, _) => {
                let expected = expected.iter().map(|kind| kind.to_string()).collect::<Vec<_>>().join("`, `");

                if expected.is_empty() {
                    "Unexpected end of file".to_string()
                } else if expected.len() == 1 {
                    format!("Expected `{}` before end of file", expected)
                } else {
                    format!("Expected one of `{}` before end of file", expected)
                }
            }
            ParseError::UnexpectedToken(expected, found, _) => {
                let expected = expected.iter().map(|kind| kind.to_string()).collect::<Vec<_>>().join("`, `");

                let found = found.to_string();

                if expected.is_empty() {
                    format!("Unexpected token `{}`", found)
                } else if expected.len() == 1 {
                    format!("Expected `{}`, found `{}`", expected, found)
                } else {
                    format!("Expected one of `{}`, found `{}`", expected, found)
                }
            }
            ParseError::UnclosedLiteralString(kind, _) => match kind {
                LiteralStringKind::SingleQuoted => "Unclosed single-quoted string".to_string(),
                LiteralStringKind::DoubleQuoted => "Unclosed double-quoted string".to_string(),
            },
        };

        write!(f, "{}", message)
    }
}

impl std::error::Error for SyntaxError {}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::SyntaxError(e) => Some(e),
            _ => None,
        }
    }
}
impl From<SyntaxError> for Issue {
    fn from(error: SyntaxError) -> Issue {
        let position = error.position();
        let span = Span::new(position, Position { offset: position.offset + 1, ..position });

        Issue::error(error.to_string()).with_annotation(Annotation::primary(span).with_message("Syntax error."))
    }
}

impl From<SyntaxError> for ParseError {
    fn from(error: SyntaxError) -> Self {
        ParseError::SyntaxError(error)
    }
}

impl From<&ParseError> for Issue {
    fn from(error: &ParseError) -> Self {
        let span = error.span();

        Issue::error(error.to_string()).with_annotation(Annotation::primary(span).with_message("Invalid syntax."))
    }
}
