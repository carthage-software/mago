use std::fs;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionsFile {
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionInfo {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub stable: bool,
    #[serde(default)]
    pub paths: Vec<String>,
}

impl VersionsFile {
    pub fn load(path: &Path, current_version: &str) -> Result<Self> {
        let mut versions = if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("failed to read versions file from {}", path.display()))?;

            serde_json::from_str::<VersionsFile>(&content).context("failed to parse versions JSON")?
        } else {
            VersionsFile { versions: Vec::new() }
        };

        if current_version.starts_with('v') && !versions.versions.iter().any(|version| version.id == current_version) {
            versions.versions.push(VersionInfo {
                id: current_version.to_string(),
                label: current_version.trim_start_matches('v').to_string(),
                stable: true,
                paths: Vec::new(),
            });
        }

        Ok(versions)
    }
}
