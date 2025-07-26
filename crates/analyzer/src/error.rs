use mago_span::Span;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AnalysisError {
    UserError(String),
    InternalError(String, Span),
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::UserError(message) => write!(f, "User Error: {message}"),
            AnalysisError::InternalError(message, span) => write!(
                f,
                "Internal Error: {} at {}-{}:{}",
                message, span.start.source.0, span.start.offset, span.end.offset
            ),
        }
    }
}

impl std::error::Error for AnalysisError {}
