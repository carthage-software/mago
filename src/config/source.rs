use std::path::PathBuf;

use config::ConfigBuilder;
use config::Value;
use config::ValueKind;
use config::builder::BuilderState;
use serde::Deserialize;
use serde::Serialize;

use crate::config::CURRENT_DIR;
use crate::config::ConfigurationEntry;
use crate::error::Error;

/// Configuration options for source discovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceConfiguration {
    /// The workspace directory from which to start scanning.
    ///
    /// Defaults to the current working directory.
    pub workspace: PathBuf,

    /// Paths to user defined source files.
    ///
    /// If empty, all files in the workspace directory are included.
    ///
    /// Defaults to `[]`.
    pub paths: Vec<PathBuf>,

    /// Paths to non-user defined files to include in the scan.
    ///
    /// Defaults to `[]`.
    pub includes: Vec<PathBuf>,

    /// Patterns to exclude from the scan.
    ///
    /// Defaults to `[]`.
    pub excludes: Vec<String>,

    /// File extensions to filter by.
    ///
    /// Defaults to `[".php"]`.
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
        Self { workspace, paths: vec![], includes: vec![], excludes: vec![], extensions: vec![] }
    }
}

impl ConfigurationEntry for SourceConfiguration {
    fn configure<St: BuilderState>(self, builder: ConfigBuilder<St>) -> Result<ConfigBuilder<St>, Error> {
        builder
            .set_default(
                "source.workspace",
                Value::new(None, ValueKind::String(self.workspace.to_string_lossy().to_string())),
            )?
            .set_default("source.paths", Value::new(None, ValueKind::Array(vec![])))?
            .set_default("source.includes", Value::new(None, ValueKind::Array(vec![])))?
            .set_default("source.excludes", Value::new(None, ValueKind::Array(vec![])))?
            .set_default(
                "source.extensions",
                Value::new(None, ValueKind::Array(vec![Value::new(None, ValueKind::String("php".to_string()))])),
            )
            .map_err(Error::from)
    }

    fn normalize(&mut self) -> Result<(), Error> {
        // Make workspace absolute if not already
        let workspace =
            if !self.workspace.is_absolute() { (*CURRENT_DIR).join(&self.workspace) } else { self.workspace.clone() };

        self.workspace = workspace.canonicalize().map_err(|e| Error::CanonicalizingPath(workspace, e))?;

        // Normalize source paths
        self.paths = self
            .paths
            .iter()
            .filter_map(|p| {
                let path = if p.is_absolute() { p.clone() } else { self.workspace.join(p) };

                path.canonicalize().ok()
            })
            .collect::<Vec<PathBuf>>();

        // Normalize include paths
        self.includes = self
            .includes
            .iter()
            .filter_map(|p| {
                let path = if p.is_absolute() { p.clone() } else { self.workspace.join(p) };

                path.canonicalize().ok()
            })
            .collect::<Vec<PathBuf>>();

        Ok(())
    }
}
