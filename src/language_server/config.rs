//! Per-server feature configuration.
//!
//! The `language-server` subcommand passes one of these to [`super::serve`]
//! to control which subsystems run, which capabilities are advertised,
//! and which `mago.toml` settings drive the analyzer, linter, and
//! formatter.

use std::sync::Arc;

use mago_analyzer::plugin::PluginRegistry;

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

    /// Pre-built analyzer plugin registry. Honours `[analyzer].plugins`
    /// and `[analyzer].disable_default_plugins`. Built once at startup
    /// because plugin lookup is lossy and the LSP doesn't reload config.
    pub plugin_registry: Arc<PluginRegistry>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            analyzer: true,
            linter: true,
            formatter: true,
            configuration: Configuration::from_workspace(std::env::current_dir().unwrap_or_default()),
            plugin_registry: Arc::new(PluginRegistry::with_library_providers()),
        }
    }
}
