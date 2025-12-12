//! Error types for the plugin system.

use std::fmt;

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

/// Errors that can occur during plugin operations.
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum PluginError {
    /// Plugin initialization failed.
    InitializationFailed { name: String, reason: String },

    /// Plugin returned an invalid result.
    InvalidResult { plugin: String, operation: String, reason: String },

    /// Plugin configuration error.
    Configuration { plugin: String, reason: String },

    /// Internal plugin error.
    Internal { reason: String },
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::InitializationFailed { name, reason } => {
                write!(f, "Plugin '{}' failed to initialize: {}", name, reason)
            }
            PluginError::InvalidResult { plugin, operation, reason } => {
                write!(f, "Plugin '{}' returned invalid result for '{}': {}", plugin, operation, reason)
            }
            PluginError::Configuration { plugin, reason } => {
                write!(f, "Plugin '{}' configuration error: {}", plugin, reason)
            }
            PluginError::Internal { reason } => {
                write!(f, "Internal plugin error: {}", reason)
            }
        }
    }
}

impl std::error::Error for PluginError {}
