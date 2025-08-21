use serde::Serialize;

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

use crate::ast::LiteralStringKind;
use crate::token::TokenKind;

const SYNTAX_ERROR_CODE: &str = "syntax";
const PARSE_ERROR_CODE: &str = "parse";

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub enum SyntaxError {
    UnexpectedToken(FileId, u8, Position),
    UnrecognizedToken(FileId, u8, Position),
    UnexpectedEndOfFile(FileId, Position),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ParseError {
    SyntaxError(SyntaxError),
    UnexpectedEndOfFile(Vec<TokenKind>, FileId, Position),
    UnexpectedToken(Vec<TokenKind>, TokenKind, Span),
    UnclosedLiteralString(LiteralStringKind, Span),
}

impl HasFileId for SyntaxError {
    fn file_id(&self) -> FileId {
        match self {
            Self::UnexpectedToken(file_id, _, _) => *file_id,
            Self::UnrecognizedToken(file_id, _, _) => *file_id,
            Self::UnexpectedEndOfFile(file_id, _) => *file_id,
        }
    }
}

impl HasFileId for ParseError {
    fn file_id(&self) -> FileId {
        match self {
            ParseError::SyntaxError(syntax_error) => syntax_error.file_id(),
            ParseError::UnexpectedEndOfFile(_, file_id, _) => *file_id,
            ParseError::UnexpectedToken(_, _, span) => span.file_id,
            ParseError::UnclosedLiteralString(_, span) => span.file_id,
        }
    }
}

impl HasSpan for SyntaxError {
    fn span(&self) -> Span {
        let (file_id, position) = match self {
            Self::UnexpectedToken(file_id, _, p) => (file_id, p),
            Self::UnrecognizedToken(file_id, _, p) => (file_id, p),
            Self::UnexpectedEndOfFile(file_id, p) => (file_id, p),
        };

        Span::new(*file_id, *position, position.forward(1))
    }
}

impl HasSpan for ParseError {
    fn span(&self) -> Span {
        match &self {
            ParseError::SyntaxError(syntax_error) => syntax_error.span(),
            ParseError::UnexpectedEndOfFile(_, file_id, position) => Span::new(*file_id, *position, *position),
            ParseError::UnexpectedToken(_, _, span) => *span,
            ParseError::UnclosedLiteralString(_, span) => *span,
        }
    }
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UnexpectedToken(_, token, _) => &format!("Unexpected token `{}` (0x{:02X})", *token as char, token),
            Self::UnrecognizedToken(_, token, _) => {
                &format!("Unrecognised token `{}` (0x{:02X})", *token as char, token)
            }
            Self::UnexpectedEndOfFile(_, _) => "Unexpected end of file",
        };

        write!(f, "{message}")
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ParseError::SyntaxError(e) => {
                return write!(f, "{e}");
            }
            ParseError::UnexpectedEndOfFile(expected, _, _) => {
                let expected = expected.iter().map(|kind| kind.to_string()).collect::<Vec<_>>().join("`, `");

                if expected.is_empty() {
                    "Unexpected end of file".to_string()
                } else if expected.len() == 1 {
                    format!("Expected `{expected}` before end of file")
                } else {
                    format!("Expected one of `{expected}` before end of file")
                }
            }
            ParseError::UnexpectedToken(expected, found, _) => {
                let expected = expected.iter().map(|kind| kind.to_string()).collect::<Vec<_>>().join("`, `");

                let found = found.to_string();

                if expected.is_empty() {
                    format!("Unexpected token `{found}`")
                } else if expected.len() == 1 {
                    format!("Expected `{expected}`, found `{found}`")
                } else {
                    format!("Expected one of `{expected}`, found `{found}`")
                }
            }
            ParseError::UnclosedLiteralString(kind, _) => match kind {
                LiteralStringKind::SingleQuoted => "Unclosed single-quoted string".to_string(),
                LiteralStringKind::DoubleQuoted => "Unclosed double-quoted string".to_string(),
            },
        };

        write!(f, "{message}")
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
impl From<&SyntaxError> for Issue {
    fn from(error: &SyntaxError) -> Issue {
        let span = error.span();

        Issue::error("Syntax error encountered during lexing")
            .with_code(SYNTAX_ERROR_CODE)
            .with_annotation(Annotation::primary(span).with_message(error.to_string()))
            .with_note("This error indicates that the lexer encountered a syntax issue.")
            .with_help("Check the syntax of your code.")
    }
}

impl From<SyntaxError> for ParseError {
    fn from(error: SyntaxError) -> Self {
        ParseError::SyntaxError(error)
    }
}

impl From<&ParseError> for Issue {
    fn from(error: &ParseError) -> Self {
        if let ParseError::SyntaxError(syntax_error) = error {
            syntax_error.into()
        } else {
            Issue::error("Fatal parse error encountered")
                .with_code(PARSE_ERROR_CODE)
                .with_annotation(Annotation::primary(error.span()).with_message(error.to_string()))
                .with_note("This error indicates that the parser encountered a parse issue.")
                .with_help("Check the syntax of your code.")
        }
    }
}
