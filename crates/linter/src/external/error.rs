use mago_extension::PayloadError;
use mago_extension::WorkerError;

/// A failure while registering or running external linter rules.
#[derive(Debug)]
pub enum ExternalLintError {
    /// Communication with an extension worker failed.
    Worker(WorkerError),
    /// A linter-domain payload was malformed or violated the protocol.
    Protocol(String),
    /// Workers in the same pool advertised different extension definitions.
    InconsistentRegistration,
    /// Two worker pools advertised the same extension identifier.
    DuplicateExtension(String),
    /// An external rule advertised an issue code already owned by another rule.
    DuplicateRule(String),
    /// A source file is too large for the linter wire protocol.
    FileTooLarge(usize),
}

impl std::fmt::Display for ExternalLintError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Worker(error) => write!(formatter, "external linter worker failed: {error}"),
            Self::Protocol(message) => write!(formatter, "external linter protocol error: {message}"),
            Self::InconsistentRegistration => {
                formatter.write_str("workers in one pool advertised different linter registrations")
            }
            Self::DuplicateExtension(identifier) => {
                write!(formatter, "multiple worker pools advertised extension `{identifier}`")
            }
            Self::DuplicateRule(code) => write!(formatter, "linter issue code `{code}` is registered more than once"),
            Self::FileTooLarge(size) => {
                write!(formatter, "source file is {size} bytes, exceeding the external linter protocol limit")
            }
        }
    }
}

impl std::error::Error for ExternalLintError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Worker(error) => Some(error),
            _ => None,
        }
    }
}

impl From<WorkerError> for ExternalLintError {
    fn from(error: WorkerError) -> Self {
        Self::Worker(error)
    }
}

impl From<PayloadError> for ExternalLintError {
    fn from(error: PayloadError) -> Self {
        Self::Protocol(error.to_string())
    }
}
