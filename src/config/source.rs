use std::path::PathBuf;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::config::CURRENT_DIR;
use crate::consts::PHP_EXTENSION;
use crate::error::Error;

/// Configuration options for source discovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SourceConfiguration {
    /// The workspace directory from which to start scanning.
    ///
    /// Defaults to the current working directory.
    pub workspace: PathBuf,

    /// Paths or glob patterns for user defined source files.
    ///
    /// Supports both directory paths (e.g., "src") and glob patterns (e.g., "src/**/*.php").
    /// If empty, all files in the workspace directory are included.
    ///
    /// Defaults to `[]`.
    #[serde(default)]
    pub paths: Vec<String>,

    /// Paths or glob patterns for non-user defined files to include in the scan.
    ///
    /// Supports both directory paths and glob patterns (same as `paths`).
    ///
    /// Defaults to `[]`.
    #[serde(default)]
    pub includes: Vec<String>,

    /// Patterns to exclude from the scan.
    ///
    /// Defaults to `[]`.
    #[serde(default)]
    pub excludes: Vec<String>,

    /// File extensions to filter by.
    ///
    /// Defaults to `[".php"]`.
    #[serde(default = "default_extensions")]
    pub extensions: Vec<String>,
}

impl SourceConfiguration {
    /// Creates a new `SourceConfiguration` with the given workspace directory.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace directory from which to start scanning.
    ///
    /// # Returns
    ///
    /// A new `SourceConfiguration` with the given workspace directory.
    pub fn from_workspace(workspace: PathBuf) -> Self {
        Self {
            workspace,
            paths: vec![],
            includes: vec![],
            excludes: vec![],
            extensions: vec![PHP_EXTENSION.to_string()],
        }
    }
}

impl SourceConfiguration {
    pub fn normalize(&mut self) -> Result<(), Error> {
        // Make workspace absolute if not already
        let workspace =
            if !self.workspace.is_absolute() { (*CURRENT_DIR).join(&self.workspace) } else { self.workspace.clone() };

        self.workspace = workspace.canonicalize().map_err(|e| Error::CanonicalizingPath(workspace, e))?;

        Ok(())
    }
}

fn default_extensions() -> Vec<String> {
    vec![PHP_EXTENSION.to_string()]
}
