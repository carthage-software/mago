//! The single-workspace [`Server`]: the transport-agnostic core that owns one
//! workspace's file database, analysis service, and per-file derived-data
//! caches, and answers editor queries against them.

use std::sync::Arc;

use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::artifacts::editor::EditorFileData;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId};
use mago_database::membership::PathExclusions;
use mago_formatter::settings::FormatSettings;
use mago_orchestrator::service::incremental_analysis::IncrementalAnalysisService;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;

use crate::capabilities::call_hierarchy::CallGraph;
use crate::error::ServerError;
use crate::file_analysis::FileAnalysis;
use crate::file_analysis::cache::FileAnalyses;
use crate::linter::LinterContext;
use crate::settings::Settings;

#[cfg(test)]
mod tests;

/// A transport-agnostic backend for a single workspace.
///
/// Owns the workspace's [`Database`], an [`IncrementalAnalysisService`], a
/// [`LinterContext`], and the per-file analysis / expression-type caches. It
/// performs no I/O: callers supply file contents as in-memory buffers and the
/// server answers queries in terms of [`FileId`]s and byte offsets.
pub struct Server {
    database: Database<'static>,
    pub(crate) tokens: crate::lookup::TokenCache,
    service: IncrementalAnalysisService,
    pub(crate) php_version: PHPVersion,
    pub(crate) formatter: FormatSettings,
    pub(crate) formatter_enabled: bool,
    analyzer_enabled: bool,
    analyzer_excludes: PathExclusions,
    pub(crate) formatter_excludes: PathExclusions,
    file_analyses: FileAnalyses,
    pub(crate) call_graph: Option<CallGraph>,
}

impl Server {
    /// Build a server for one workspace from an already-loaded file database,
    /// decoded codebase metadata, and resolved [`Settings`].
    ///
    /// Construction performs no analysis; call [`Server::analyze`] for the
    /// initial pass.
    #[must_use]
    pub fn new(
        database: Database<'static>,
        metadata: CodebaseMetadata,
        symbol_references: SymbolReferences,
        settings: Settings,
    ) -> Self {
        let Settings { php_version, features, mut parser, analyzer, linter, formatter, plugin_registry, excludes } =
            settings;
        parser.recover_incomplete = true;

        let mut service = IncrementalAnalysisService::new(
            database.read_only(),
            metadata,
            symbol_references,
            analyzer,
            parser,
            plugin_registry,
        );
        service.set_editor_artifacts(features.analyzer);
        service.set_scan_only(!features.analyzer);

        let linter = LinterContext::new(linter, parser, features.linter).with_excludes(excludes.linter);
        let file_analyses = FileAnalyses::new(linter, features.analyzer, excludes.analyzer.clone());

        Self {
            database,
            tokens: crate::lookup::TokenCache::new(parser.lexer),
            service,
            php_version,
            formatter,
            formatter_enabled: features.formatter,
            analyzer_enabled: features.analyzer,
            analyzer_excludes: excludes.analyzer,
            formatter_excludes: excludes.formatter,
            file_analyses,
            call_graph: None,
        }
    }

    /// Borrow the workspace file database.
    #[must_use]
    pub fn database(&self) -> &Database<'static> {
        &self.database
    }

    /// Builtin and plugin-owned sources are read-only; host buffers need no disk path.
    pub(crate) fn editable_file(&self, file: FileId) -> Option<Arc<File>> {
        self.database.get(&file).ok().filter(|file| !file.file_type.is_builtin() && !file.file_type.is_external())
    }

    /// Check every target before returning a batch of edits.
    pub(crate) fn can_edit_files(&self, files: impl IntoIterator<Item = FileId>) -> bool {
        files.into_iter().all(|file| self.editable_file(file).is_some())
    }

    pub(crate) fn parser_settings(&self) -> mago_syntax::settings::ParserSettings {
        self.file_analyses.parser_settings()
    }

    /// Mutably borrow the workspace file database (to add / update / delete
    /// files as the editor's buffers change).
    pub fn database_mut(&mut self) -> &mut Database<'static> {
        self.call_graph = None;
        &mut self.database
    }

    /// Borrow the populated codebase metadata (symbol table).
    #[must_use]
    pub fn codebase(&self) -> &CodebaseMetadata {
        self.service.codebase()
    }

    /// Whether a complete semantic analysis is available for editor queries.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.service.is_initialized()
    }

    /// The issues from the most recent analysis pass, if any.
    #[must_use]
    pub fn last_issues(&self) -> Option<IssueCollection> {
        self.service.last_issues()
    }

    /// Borrows cached analyzer issues from the last analysis pass.
    pub fn analyzer_issues(&self) -> impl Iterator<Item = &IssueCollection> + '_ {
        self.service.issue_collections().filter(|_| self.analyzer_enabled)
    }

    /// Whether analyzer diagnostics and fixes are withheld from `file`.
    #[must_use]
    pub fn excludes_analyzer_issues(&self, file: FileId) -> bool {
        !self.analyzer_excludes.is_empty()
            && self.database.get_ref(&file).is_ok_and(|file| self.analyzer_excludes.excludes_file(file))
    }

    /// Whether host files need a file pass to publish diagnostics.
    #[must_use]
    pub fn reports_file_diagnostics(&self) -> bool {
        self.file_analyses.reports_any()
    }

    /// Shares analyzer issue sets, keyed by owning file and issue source.
    pub fn shared_analyzer_issues(&self) -> impl Iterator<Item = (Option<FileId>, bool, &Arc<IssueCollection>)> + '_ {
        self.service.shared_issue_collections().filter(|_| self.analyzer_enabled)
    }

    /// Run a full analysis pass over the workspace. Returns an empty result
    /// when the analyzer is disabled.
    ///
    /// # Errors
    ///
    /// Returns [`ServerError`] if the underlying analysis fails.
    pub fn analyze(&mut self) -> Result<AnalysisResult, ServerError> {
        self.prepare_analysis();
        let result = self.service.analyze()?;
        if self.analyzer_enabled { Ok(result) } else { Ok(AnalysisResult::new(SymbolReferences::new())) }
    }

    /// Runs a full analysis without copying the cached issues.
    ///
    /// Read the results through [`analyzer_issues()`](Self::analyzer_issues).
    ///
    /// # Errors
    ///
    /// Returns [`ServerError`] if analysis fails.
    pub fn analyze_without_collecting(&mut self) -> Result<(), ServerError> {
        self.prepare_analysis();
        self.service.analyze_without_collecting()?;
        Ok(())
    }

    /// Refresh the analysis service's view of the database, then re-analyze the
    /// `changed` files. Returns an empty result when the analyzer is disabled.
    ///
    /// # Errors
    ///
    /// Returns [`ServerError`] if the underlying analysis fails.
    pub fn analyze_incremental(&mut self, changed: &[FileId]) -> Result<AnalysisResult, ServerError> {
        self.prepare_analysis();
        let result = self.service.analyze_incremental(Some(changed))?;
        if self.analyzer_enabled { Ok(result) } else { Ok(AnalysisResult::new(SymbolReferences::new())) }
    }

    /// Refreshes the database and analyzes changed files without copying cached issues.
    ///
    /// Read the results through [`analyzer_issues()`](Self::analyzer_issues).
    ///
    /// # Errors
    ///
    /// Returns [`ServerError`] if analysis fails.
    pub fn analyze_incremental_without_collecting(&mut self, changed: &[FileId]) -> Result<(), ServerError> {
        self.prepare_analysis();
        self.service.analyze_incremental_without_collecting(Some(changed))?;
        Ok(())
    }

    fn prepare_analysis(&mut self) {
        self.call_graph = None;
        self.service.update_database(self.database.read_only());
    }

    /// Analyzes changed files and refreshes their editor caches after a database update.
    ///
    /// # Errors
    ///
    /// Returns [`ServerError`] if analysis fails.
    pub fn refresh_changed_files(&mut self, changed: &[FileId]) -> Result<(), ServerError> {
        self.invalidate_artifacts(changed);
        self.tokens.invalidate(changed.iter().copied());
        let result = self.analyze_incremental_without_collecting(changed);
        self.file_analyses.refresh_diagnostics(&self.database, changed);
        result
    }

    /// Editor data from the last analysis. Queries never trigger another analysis.
    pub fn editor_file(&self, file_id: FileId) -> Option<Arc<EditorFileData>> {
        self.service.editor_file(file_id).map(Arc::clone)
    }

    /// Drop cached derived data for the given files.
    pub fn invalidate_artifacts(&mut self, files: &[FileId]) {
        self.call_graph = None;
        self.file_analyses.invalidate(files);
    }

    /// Return the [`FileAnalysis`] for `file_id`, building it on a cache miss.
    /// One parse + resolve per content hash, shared across capability queries.
    pub fn file_analysis_for(&mut self, file_id: FileId) -> Option<Arc<FileAnalysis>> {
        let file = self.database.get(&file_id).ok()?;
        Some(self.file_analyses.get_or_build(&file))
    }

    /// Build (or rebuild) the analysis for every changed host file.
    pub fn refresh_analyses(&mut self, file_ids: &[FileId]) {
        self.call_graph = None;
        self.file_analyses.refresh(&self.database, file_ids);
    }

    /// Build analyses for every host file whose diagnostics come from the file pass.
    pub fn refresh_all_host_analyses(&mut self) {
        self.call_graph = None;
        self.file_analyses.refresh_all(&self.database);
    }

    /// Iterate the cached lint issues across every analyzed file.
    pub fn lint_issues(&self) -> impl Iterator<Item = &IssueCollection> + '_ {
        self.file_analyses.analyses().map(|analysis| &analysis.lint_issues)
    }

    /// Shares lint issue owners so clients can detect unchanged results.
    pub fn lint_issue_sources(&self) -> impl Iterator<Item = (FileId, &Arc<FileAnalysis>)> + '_ {
        self.file_analyses.sources()
    }

    /// Iterate every cached per-file analysis.
    pub fn analyses(&self) -> impl Iterator<Item = &Arc<FileAnalysis>> + '_ {
        self.file_analyses.analyses()
    }
}
