use mago_extension::PayloadError;
use mago_extension::WorkerError;

#[derive(Debug)]
pub enum ExternalAnalyzerError {
    Worker(WorkerError),
    Protocol(String),
    InconsistentRegistration,
    InconsistentInitialization,
    DuplicateExtension(String),
    DuplicatePlugin(String),
}

impl ExternalAnalyzerError {
    #[must_use]
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol(message.into())
    }
}

impl std::fmt::Display for ExternalAnalyzerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Worker(error) => write!(formatter, "external analyzer worker failed: {error}"),
            Self::Protocol(message) => write!(formatter, "external analyzer protocol error: {message}"),
            Self::InconsistentRegistration => {
                formatter.write_str("workers in an extension pool advertised different analyzer registrations")
            }
            Self::InconsistentInitialization => {
                formatter.write_str("workers in an extension pool produced different analyzer initialization stubs")
            }
            Self::DuplicateExtension(identifier) => {
                write!(formatter, "external extension `{identifier}` is registered by more than one host")
            }
            Self::DuplicatePlugin(identifier) => {
                write!(formatter, "external analyzer plugin `{identifier}` is registered more than once")
            }
        }
    }
}

impl std::error::Error for ExternalAnalyzerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Worker(error) => Some(error),
            _ => None,
        }
    }
}

impl From<WorkerError> for ExternalAnalyzerError {
    fn from(error: WorkerError) -> Self {
        Self::Worker(error)
    }
}

impl From<PayloadError> for ExternalAnalyzerError {
    fn from(error: PayloadError) -> Self {
        Self::Protocol(error.to_string())
    }
}

pub(super) fn protocol(message: impl Into<String>) -> ExternalAnalyzerError {
    ExternalAnalyzerError::Protocol(message.into())
}
