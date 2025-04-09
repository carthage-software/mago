use serde::Serialize;

use mago_span::Position;
use mago_span::Span;

use crate::token::TypeTokenKind;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub enum SyntaxError {
    UnexpectedToken(u8, Position),
    UnrecognizedToken(u8, Position),
    UnexpectedEndOfFile(Position),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ParseError {
    SyntaxError(SyntaxError),
    UnexpectedEndOfFile(Vec<TypeTokenKind>, Position),
    UnexpectedToken(Vec<TypeTokenKind>, TypeTokenKind, Span),
    UnclosedLiteralString(Span),
}

impl From<SyntaxError> for ParseError {
    fn from(error: SyntaxError) -> Self {
        ParseError::SyntaxError(error)
    }
}
