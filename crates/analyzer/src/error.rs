use mago_span::Span;

use crate::plugin::PluginError;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AnalysisError {
    Plugin(PluginError),
    UserError(String),
    InternalError(String, Span),
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::Plugin(err) => write!(f, "Plugin Error: {}", err),
            AnalysisError::UserError(message) => write!(f, "User Error: {message}"),
            AnalysisError::InternalError(message, span) => {
                write!(f, "Internal Error: {} at {}-{}:{}", message, span.file_id, span.start.offset, span.end.offset)
            }
        }
    }
}

impl std::error::Error for AnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AnalysisError::Plugin(err) => Some(err),
            _ => None,
        }
    }
}

impl From<PluginError> for AnalysisError {
    fn from(err: PluginError) -> Self {
        AnalysisError::Plugin(err)
    }
}
