//! Per-server feature configuration.
//!
//! The `language-server` subcommand passes one of these to [`super::serve`]
//! to control which subsystems run and which capabilities are advertised.
//! Disabling the analyzer also disables every capability that depends on
//! analyzer-derived metadata or per-expression types; see [`ServerConfig`]
//! field docs.

/// Runtime feature switches for the language server.
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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { analyzer: true, linter: true, formatter: true }
    }
}
