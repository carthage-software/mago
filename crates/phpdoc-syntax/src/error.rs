use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize)]
pub enum ParseError {
    UnexpectedToken(Span),
    UnexpectedEndOfInput(Span),
}

impl ParseError {
    #[must_use]
    pub fn note(&self) -> String {
        match self {
            ParseError::UnexpectedToken(_) => {
                "The parser encountered a token that is not valid at this position.".to_string()
            }
            ParseError::UnexpectedEndOfInput(_) => {
                "The PHPDoc comment ended before the construct was complete.".to_string()
            }
        }
    }

    #[must_use]
    pub fn help(&self) -> String {
        match self {
            ParseError::UnexpectedToken(_) => "Review the PHPDoc syntax near the unexpected token.".to_string(),
            ParseError::UnexpectedEndOfInput(_) => {
                "Complete the construct; check for unclosed `<>`, `()`, `[]`, or `{}`.".to_string()
            }
        }
    }
}

impl HasSpan for ParseError {
    fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken(span) | ParseError::UnexpectedEndOfInput(span) => *span,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(_) => write!(f, "Unexpected token in PHPDoc"),
            ParseError::UnexpectedEndOfInput(_) => write!(f, "Unexpected end of PHPDoc input"),
        }
    }
}

impl std::error::Error for ParseError {}
