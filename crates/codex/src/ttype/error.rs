use mago_span::HasSpan;
use mago_span::Span;
use serde::Serialize;

use mago_type_syntax::error::ParseError;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum TypeError {
    ParseError(ParseError),
    UnsupportedType(String, Span),
    InvalidType(String, Span),
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::ParseError(err) => write!(f, "{err}"),
            TypeError::UnsupportedType(ty, _) => {
                write!(f, "Unsupported type syntax: `{ty}`")
            }
            TypeError::InvalidType(message, _) => {
                write!(f, "{message}")
            }
        }
    }
}

impl TypeError {
    pub fn note(&self) -> String {
        match self {
            TypeError::ParseError(err) => err.note(),
            TypeError::UnsupportedType(ty, _) => {
                format!("The type `{ty}` is syntactically valid but is not yet supported by the analyzer.")
            }
            TypeError::InvalidType(ty, _) => {
                format!("The type declaration `{ty}` is not valid or could not be resolved.")
            }
        }
    }

    pub fn help(&self) -> String {
        match self {
            TypeError::ParseError(err) => err.help(),
            TypeError::UnsupportedType(_, _) => "Try using a simpler or more standard type declaration.".to_string(),
            TypeError::InvalidType(_, _) => {
                "Check for typos or ensure the type is a valid class, interface, or built-in type.".to_string()
            }
        }
    }
}

// Ensure HasSpan is implemented for TypeError to get the location of the error
impl HasSpan for TypeError {
    fn span(&self) -> Span {
        match self {
            TypeError::ParseError(err) => err.span(),
            TypeError::UnsupportedType(_, span) | TypeError::InvalidType(_, span) => *span,
        }
    }
}

impl From<ParseError> for TypeError {
    fn from(err: ParseError) -> Self {
        TypeError::ParseError(err)
    }
}
