use bincode::error::DecodeError;
use bincode::error::EncodeError;

#[derive(Debug)]
pub enum PreludeError {
    Encode(EncodeError),
    Decode(DecodeError),
}

impl std::error::Error for PreludeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PreludeError::Encode(err) => Some(err),
            PreludeError::Decode(err) => Some(err),
        }
    }
}

impl std::fmt::Display for PreludeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreludeError::Encode(err) => write!(f, "Prelude encoding error: {err}"),
            PreludeError::Decode(err) => write!(f, "Prelude decoding error: {err}"),
        }
    }
}

impl From<DecodeError> for PreludeError {
    fn from(err: DecodeError) -> Self {
        PreludeError::Decode(err)
    }
}

impl From<EncodeError> for PreludeError {
    fn from(err: EncodeError) -> Self {
        PreludeError::Encode(err)
    }
}
