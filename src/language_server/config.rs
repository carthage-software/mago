//! Per-server feature configuration.
//!
//! The `language-server` subcommand passes one of these to [`super::serve`]
//! to control which subsystems run, which capabilities are advertised,
//! and which `mago.toml` settings drive the analyzer, linter, and
//! formatter.

use std::path::PathBuf;

use crate::config::Configuration;

/// Runtime feature switches plus the user's `mago.toml`. Subsystem
/// settings (analyzer, linter, formatter, parser, source paths) are
/// read directly from `configuration` at the points that need them.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Run the static analyzer on each change. Required for the per-file
    /// `CodebaseMetadata` and `AnalysisArtifacts` we use for hover types,
    /// member completion, signature help, inlay hints, references via
    /// FQCN, definitions for non-class symbols, and code-lens reference
    /// counts. With this off all of those capabilities go quiet.
    pub analyzer: bool,

    /// Run the linter on each change. Linter issues are merged into the
    /// `publishDiagnostics` stream and their `edits` show up as
    /// `quickfix` code actions. Independent of the analyzer.
    pub linter: bool,

    /// Advertise and serve `textDocument/formatting`. Independent of
    /// analyzer + linter.
    pub formatter: bool,

    /// Loaded `mago.toml` (or default). Drives PHP version, parser /
    /// analyzer / linter / formatter settings, and the source-discovery
    /// patterns the LSP feeds into the database loader.
    pub configuration: Configuration,

    /// Workspace directory from an explicit `--workspace` flag. When set it
    /// takes precedence over the client's `rootUri`, so the bootstrap honours
    /// the same root as `mago lint` / `mago analyze`. `None` when the flag was
    /// omitted, in which case the client-provided root is used.
    pub workspace_override: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            analyzer: true,
            linter: true,
            formatter: true,
            configuration: Configuration::from_workspace(std::env::current_dir().unwrap_or_default()),
            workspace_override: None,
        }
    }
}
