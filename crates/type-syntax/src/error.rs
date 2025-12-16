use serde::Serialize;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

use crate::token::TypeTokenKind;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub enum SyntaxError {
    UnexpectedToken(FileId, u8, Position),
    UnrecognizedToken(FileId, u8, Position),
    UnexpectedEndOfFile(FileId, Position),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ParseError {
    SyntaxError(SyntaxError),
    UnexpectedEndOfFile(FileId, Vec<TypeTokenKind>, Position),
    UnexpectedToken(Vec<TypeTokenKind>, TypeTokenKind, Span),
    UnclosedLiteralString(Span),
}

impl ParseError {
    /// Provides a detailed, user-friendly note explaining the context of the parse error.
    #[must_use]
    pub fn note(&self) -> String {
        match self {
            ParseError::SyntaxError(SyntaxError::UnrecognizedToken(_, _, _)) => {
                "An invalid character was found that is not part of any valid type syntax.".to_string()
            }
            ParseError::SyntaxError(_) => {
                "A low-level syntax error occurred while parsing the type string.".to_string()
            }
            ParseError::UnexpectedEndOfFile(_, expected, _) => {
                if expected.is_empty() {
                    "The type declaration ended prematurely.".to_string()
                } else {
                    let expected_str = expected.iter().map(|t| format!("`{t}`")).collect::<Vec<_>>().join(" or ");
                    format!("The parser reached the end of the input but expected one of: {expected_str}.")
                }
            }
            ParseError::UnexpectedToken(expected, _, _) => {
                if expected.is_empty() {
                    "The parser encountered a token that was not expected at this position.".to_string()
                } else {
                    let expected_str = expected.iter().map(|t| format!("`{t}`")).collect::<Vec<_>>().join(" or ");
                    format!("The parser expected one of the following here: {expected_str}.")
                }
            }
            ParseError::UnclosedLiteralString(_) => {
                "String literals within type declarations must be closed with a matching quote.".to_string()
            }
        }
    }

    /// Provides a concise, actionable help message suggesting a fix for the error.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            ParseError::SyntaxError(SyntaxError::UnrecognizedToken(_, _, _)) => {
                "Remove or replace the invalid character.".to_string()
            }
            ParseError::SyntaxError(_) => "Review the syntax of the type declaration for errors.".to_string(),
            ParseError::UnexpectedEndOfFile(_, _, _) => {
                "Complete the type declaration. Check for unclosed parentheses `()`, angle brackets `<>`, or curly braces `{}`.".to_string()
            }
            ParseError::UnexpectedToken(_, _, _) => {
                "Review the type syntax near the unexpected token.".to_string()
            }
            ParseError::UnclosedLiteralString(_) => {
                "Add a closing quote (`'` or `\"`) to complete the string literal.".to_string()
            }
        }
    }
}

impl HasSpan for SyntaxError {
    fn span(&self) -> Span {
        let (file_id, position) = match self {
            SyntaxError::UnexpectedToken(file_id, _, position) => (*file_id, *position),
            SyntaxError::UnrecognizedToken(file_id, _, position) => (*file_id, *position),
            SyntaxError::UnexpectedEndOfFile(file_id, position) => (*file_id, *position),
        };

        Span::new(file_id, position, position)
    }
}

impl HasSpan for ParseError {
    fn span(&self) -> Span {
        match self {
            ParseError::SyntaxError(error) => error.span(),
            ParseError::UnexpectedEndOfFile(file_id, _, position) => Span::new(*file_id, *position, *position),
            ParseError::UnexpectedToken(_, _, span) => *span,
            ParseError::UnclosedLiteralString(span) => *span,
        }
    }
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxError::UnexpectedToken(_, token, _) => {
                write!(f, "Unexpected character '{}'", *token as char)
            }
            SyntaxError::UnrecognizedToken(_, token, _) => {
                write!(f, "Unrecognized character '{}'", *token as char)
            }
            SyntaxError::UnexpectedEndOfFile(_, _) => {
                write!(f, "Unexpected end of input")
            }
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::SyntaxError(err) => write!(f, "{err}"),
            ParseError::UnexpectedEndOfFile(_, _, _) => {
                write!(f, "Unexpected end of type declaration")
            }
            ParseError::UnexpectedToken(_, token, _) => {
                write!(f, "Unexpected token `{token}`")
            }
            ParseError::UnclosedLiteralString(_) => {
                write!(f, "Unclosed string literal in type")
            }
        }
    }
}

impl std::error::Error for SyntaxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::SyntaxError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<SyntaxError> for ParseError {
    fn from(error: SyntaxError) -> Self {
        ParseError::SyntaxError(error)
    }
}
