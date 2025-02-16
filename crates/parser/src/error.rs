use serde::Serialize;

use mago_ast::ast::*;
use mago_lexer::error::SyntaxError;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;
use mago_token::TokenKind;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
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

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::SyntaxError(e) => Some(e),
            _ => None,
        }
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
