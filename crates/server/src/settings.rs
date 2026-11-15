//! The curated inputs a [`Server`](crate::Server) needs to operate.
//!
//! The server takes a [`Settings`]; **not** the CLI's `Configuration`. The
//! caller is responsible for loading configuration from disk (or building it in
//! memory) and projecting it down to the specific, already-resolved settings
//! each subsystem requires. The server never reads configuration itself.

use std::sync::Arc;

use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings as AnalyzerSettings;
use mago_database::membership::PathExclusions;
use mago_formatter::settings::FormatSettings;
use mago_linter::settings::Settings as LinterSettings;
use mago_php_version::PHPVersion;
use mago_syntax::settings::ParserSettings;

/// Which subsystems run for a workspace.
///
/// These switch behaviour, not capability advertisement; deciding which LSP
/// capabilities to expose is the protocol layer's concern. With the analyzer
/// off, the per-file type information that powers hover types, member
/// completion, signature help, inlay hints, and reference counts is absent, so
/// those answers degrade or go quiet.
#[derive(Debug, Clone, Copy)]
pub struct Features {
    /// Run the static analyzer on each change.
    pub analyzer: bool,
    /// Run the linter on each change and merge its issues into diagnostics.
    pub linter: bool,
    /// Serve formatting requests.
    pub formatter: bool,
}

/// The resolved settings a [`Server`](crate::Server) runs against.
///
/// Supplied by the caller alongside the file
/// [`Database`](mago_database::Database) and decoded codebase metadata passed
/// to [`Server::new`](crate::Server::new).
#[derive(Debug, Clone)]
pub struct Settings {
    /// The PHP language version every subsystem targets. Must match
    /// [`AnalyzerSettings::version`].
    pub php_version: PHPVersion,
    /// Which subsystems are active.
    pub features: Features,
    /// Settings for parsing PHP source into an AST.
    pub parser: ParserSettings,
    /// Settings for the static analyzer.
    pub analyzer: AnalyzerSettings,
    /// Settings for the linter.
    pub linter: LinterSettings,
    /// Settings for the formatter.
    pub formatter: FormatSettings,
    /// The analyzer plugin registry, built once by the caller. Shared so it can
    /// be reused across workspaces.
    pub plugin_registry: Arc<PluginRegistry>,
    /// Files each subsystem leaves alone.
    pub excludes: Excludes,
}

/// Host files kept out of one subsystem, like the CLI's per-tool `excludes`.
///
/// Excluded files stay in the workspace: queries still navigate them, and the
/// analyzer still infers their types. Only that subsystem's output is withheld.
#[derive(Debug, Clone, Default)]
pub struct Excludes {
    /// No analyzer diagnostics or fixes.
    pub analyzer: PathExclusions,
    /// No lint rules or fixes.
    pub linter: PathExclusions,
    /// No formatting edits.
    pub formatter: PathExclusions,
}
