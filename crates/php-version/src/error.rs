use std::error::Error;
use std::num::ParseIntError;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParsingError {
    InvalidFormat,
    ParseIntError(ParseIntError),
}

impl std::fmt::Display for ParsingError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "Invalid version format, expected 'major.minor.patch'."),
            Self::ParseIntError(e) => write!(f, "Failed to parse integer component of version: {e}."),
        }
    }
}

impl Error for ParsingError {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ParseIntError(e) => Some(e),
            Self::InvalidFormat => None,
        }
    }
}

impl From<ParseIntError> for ParsingError {
    #[inline]
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}
