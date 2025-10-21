//! Error types for the orchestrator crate.

use mago_analyzer::error::AnalysisError;
use mago_database::error::DatabaseError;

/// Errors that can occur during orchestration operations.
#[derive(Debug)]
pub enum OrchestratorError {
    /// An error occurred while accessing the database.
    Database(DatabaseError),
    /// An error occurred during analysis.
    Analysis(AnalysisError),
    /// A general error with a message.
    General(String),
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "Database error: {}", error),
            Self::Analysis(error) => write!(f, "Analysis error: {}", error),
            Self::General(message) => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for OrchestratorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Analysis(error) => Some(error),
            Self::General(_) => None,
        }
    }
}

impl From<DatabaseError> for OrchestratorError {
    fn from(error: DatabaseError) -> Self {
        Self::Database(error)
    }
}

impl From<AnalysisError> for OrchestratorError {
    fn from(error: AnalysisError) -> Self {
        Self::Analysis(error)
    }
}
