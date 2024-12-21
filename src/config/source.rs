use std::path::PathBuf;

use config::builder::BuilderState;
use config::ConfigBuilder;
use config::Value;
use config::ValueKind;
use serde::Deserialize;
use serde::Serialize;

use crate::config::ConfigurationEntry;
use crate::config::CURRENT_DIR;
use crate::error::Error;

/// Configuration options for source discovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceConfiguration {
    /// The root directory from which to start scanning.
    ///
    /// Defaults to the current working directory.
    pub root: PathBuf,

    /// Paths to user defined source files.
    ///
    /// If empty, all files in the root directory are included.
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
    /// Creates a new `SourceConfiguration` with the given root directory.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory from which to start scanning.
    ///
    /// # Returns
    ///
    /// A new `SourceConfiguration` with the given root directory.
    pub fn from_root(root: PathBuf) -> Self {
        Self { root, paths: vec![], includes: vec![], excludes: vec![], extensions: vec![] }
    }
}

impl ConfigurationEntry for SourceConfiguration {
    fn configure<St: BuilderState>(self, builder: ConfigBuilder<St>) -> Result<ConfigBuilder<St>, Error> {
        builder
            .set_default("source.root", Value::new(None, ValueKind::String(self.root.to_string_lossy().to_string())))?
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
        // Make root absolute if not already
        let root = if !self.root.is_absolute() { (*CURRENT_DIR).join(&self.root) } else { self.root.clone() };

        self.root = root.canonicalize().map_err(|e| Error::CanonicalizingPath(root, e))?;

        // Normalize source paths
        self.paths = self
            .paths
            .iter()
            .map(|p| {
                let path = if p.is_absolute() { p.clone() } else { self.root.join(p) };

                path.canonicalize().map_err(|e| Error::CanonicalizingPath(p.clone(), e))
            })
            .collect::<Result<Vec<PathBuf>, Error>>()?;

        // Normalize include paths
        self.includes = self
            .includes
            .iter()
            .map(|p| {
                let path = if p.is_absolute() { p.clone() } else { self.root.join(p) };

                path.canonicalize().map_err(|e| Error::CanonicalizingPath(p.clone(), e))
            })
            .collect::<Result<Vec<PathBuf>, Error>>()?;

        Ok(())
    }
}
