//! The single-workspace [`Server`]: the transport-agnostic core that owns one
//! workspace's file database, analysis service, and per-file derived-data
//! caches, and answers editor queries against them.

use std::sync::Arc;

use foldhash::HashMap;
use xxhash_rust::xxh3;

use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::artifacts::AnalysisArtifacts;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_formatter::settings::FormatSettings;
use mago_orchestrator::service::incremental_analysis::IncrementalAnalysisService;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_word::Word;

use crate::error::ServerError;
use crate::file_analysis;
use crate::file_analysis::FileAnalysis;
use crate::linter::LinterContext;
use crate::settings::Settings;

/// Per-file index of the named-object types inferred for each expression span,
/// derived from the analyzer's [`AnalysisArtifacts`]. Powers member completion.
#[derive(Debug, Default, Clone)]
pub struct ExpressionTypeIndex {
    /// Maps an expression's `(start, end)` byte span to the class-like names its
    /// inferred type resolves to.
    pub by_span: HashMap<(u32, u32), Vec<Word>>,
}

/// A transport-agnostic backend for a single workspace.
///
/// Owns the workspace's [`Database`], an [`IncrementalAnalysisService`], a
/// [`LinterContext`], and the per-file analysis / expression-type caches. It
/// performs no I/O: callers supply file contents as in-memory buffers and the
/// server answers queries in terms of [`FileId`]s and byte offsets.
pub struct Server {
    database: Database<'static>,
    service: IncrementalAnalysisService,
    linter: LinterContext,
    pub(crate) php_version: PHPVersion,
    pub(crate) formatter: FormatSettings,
    analyzer_enabled: bool,
    /// Per-file parse + resolve + lint results, keyed by content hash.
    file_analyses: HashMap<FileId, (u64, Arc<FileAnalysis>)>,
    /// Per-file expression-type index, keyed by content hash.
    artifact_cache: HashMap<FileId, (u64, ExpressionTypeIndex)>,
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
        let Settings { php_version, features, parser, analyzer, linter, formatter, plugin_registry } = settings;

        let service = IncrementalAnalysisService::new(
            database.read_only(),
            metadata,
            symbol_references,
            analyzer,
            parser,
            plugin_registry,
        );

        let linter = LinterContext::new(linter, parser);

        Self {
            database,
            service,
            linter,
            php_version,
            formatter,
            analyzer_enabled: features.analyzer,
            file_analyses: HashMap::default(),
            artifact_cache: HashMap::default(),
        }
    }

    /// Borrow the workspace file database.
    #[must_use]
    pub fn database(&self) -> &Database<'static> {
        &self.database
    }

    /// Mutably borrow the workspace file database (to add / update / delete
    /// files as the editor's buffers change).
    pub fn database_mut(&mut self) -> &mut Database<'static> {
        &mut self.database
    }

    /// Borrow the populated codebase metadata (symbol table).
    #[must_use]
    pub fn codebase(&self) -> &CodebaseMetadata {
        self.service.codebase()
    }

    /// The issues from the most recent analysis pass, if any.
    #[must_use]
    pub fn last_issues(&self) -> Option<IssueCollection> {
        self.service.last_issues()
    }

    /// Run a full analysis pass over the workspace. Returns an empty result
    /// when the analyzer is disabled.
    ///
    /// # Errors
    ///
    /// Returns [`ServerError`] if the underlying analysis fails.
    pub fn analyze(&mut self) -> Result<AnalysisResult, ServerError> {
        if self.analyzer_enabled {
            Ok(self.service.analyze()?)
        } else {
            Ok(AnalysisResult::new(SymbolReferences::new()))
        }
    }

    /// Refresh the analysis service's view of the database, then re-analyze the
    /// `changed` files. Returns an empty result when the analyzer is disabled.
    ///
    /// # Errors
    ///
    /// Returns [`ServerError`] if the underlying analysis fails.
    pub fn analyze_incremental(&mut self, changed: &[FileId]) -> Result<AnalysisResult, ServerError> {
        self.service.update_database(self.database.read_only());
        if self.analyzer_enabled {
            Ok(self.service.analyze_incremental(Some(changed))?)
        } else {
            Ok(AnalysisResult::new(SymbolReferences::new()))
        }
    }

    /// The expression-type index for `file_id`, building and caching it on a
    /// miss. Returns `None` when the analyzer is disabled.
    pub fn type_index_for(&mut self, file_id: FileId) -> Option<&ExpressionTypeIndex> {
        if !self.analyzer_enabled {
            return None;
        }

        let file = self.database.get(&file_id).ok()?;
        let hash = xxh3::xxh3_64(&file.contents);

        if let Some((cached_hash, _)) = self.artifact_cache.get(&file_id)
            && *cached_hash == hash
        {
            return self.artifact_cache.get(&file_id).map(|(_, idx)| idx);
        }

        let (_, artifacts) = self.service.analyze_file_with_artifacts(file_id)?;
        let index = build_index(&artifacts);
        self.artifact_cache.insert(file_id, (hash, index));
        self.artifact_cache.get(&file_id).map(|(_, idx)| idx)
    }

    /// Drop cached derived data for the given files.
    pub fn invalidate_artifacts(&mut self, files: &[FileId]) {
        for id in files {
            self.artifact_cache.remove(id);
            self.file_analyses.remove(id);
        }
    }

    /// Return the [`FileAnalysis`] for `file_id`, building it on a cache miss.
    /// One parse + resolve per content hash, shared across capability queries.
    pub fn file_analysis_for(&mut self, file_id: FileId) -> Option<Arc<FileAnalysis>> {
        let file = self.database.get(&file_id).ok()?;
        let hash = xxh3::xxh3_64(&file.contents);

        if let Some((cached_hash, analysis)) = self.file_analyses.get(&file_id)
            && *cached_hash == hash
        {
            return Some(Arc::clone(analysis));
        }

        let with_semantics = !self.analyzer_enabled;
        let analysis = Arc::new(file_analysis::build(&file, &self.linter, with_semantics));
        self.file_analyses.insert(file_id, (hash, Arc::clone(&analysis)));
        Some(analysis)
    }

    /// Build (or rebuild) the analysis for every changed host file.
    pub fn refresh_analyses(&mut self, file_ids: &[FileId]) {
        let with_semantics = !self.analyzer_enabled;
        for &file_id in file_ids {
            let Ok(file) = self.database.get(&file_id) else {
                self.file_analyses.remove(&file_id);
                continue;
            };

            if file.file_type != FileType::Host {
                continue;
            }

            let hash = xxh3::xxh3_64(&file.contents);
            let analysis = Arc::new(file_analysis::build(&file, &self.linter, with_semantics));
            self.file_analyses.insert(file_id, (hash, analysis));
        }
    }

    /// Build analyses for every host file in the database.
    pub fn refresh_all_host_analyses(&mut self) {
        let host_ids: Vec<FileId> =
            self.database.files().filter(|f| matches!(f.file_type, FileType::Host)).map(|f| f.id).collect();

        self.refresh_analyses(&host_ids);
    }

    /// Iterate the cached lint issues across every analyzed file.
    pub fn lint_issues(&self) -> impl Iterator<Item = &IssueCollection> + '_ {
        self.file_analyses.values().map(|(_, analysis)| &analysis.lint_issues)
    }

    /// Iterate every cached per-file analysis.
    pub fn analyses(&self) -> impl Iterator<Item = &Arc<FileAnalysis>> + '_ {
        self.file_analyses.values().map(|(_, analysis)| analysis)
    }
}

fn build_index(artifacts: &AnalysisArtifacts) -> ExpressionTypeIndex {
    let mut by_span: HashMap<(u32, u32), Vec<Word>> = HashMap::default();
    for (span, ty) in artifacts.expression_types.iter() {
        let mut classes: Vec<Word> = Vec::new();
        for atomic in ty.types.iter() {
            match atomic {
                TAtomic::Object(TObject::Named(n)) => classes.push(n.name),
                TAtomic::Object(TObject::Enum(e)) => classes.push(e.name),
                _ => {}
            }
        }

        if !classes.is_empty() {
            by_span.insert(*span, classes);
        }
    }

    ExpressionTypeIndex { by_span }
}
