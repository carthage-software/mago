//! Backend state lifecycle.
//!
//! On startup the backend is [`BackendState::Uninitialized`]. The
//! `initialize` request transitions it to [`BackendState::Pending`]; the
//! `initialized` notification triggers a one-shot bootstrap that walks
//! the workspace, builds the database, runs an initial full analysis (if
//! enabled by the [`super::ServerConfig`]), and transitions to
//! [`BackendState::Ready`].

use std::path::PathBuf;
use std::sync::Arc;

use foldhash::HashMap;
use tower_lsp::lsp_types::Diagnostic;
use tower_lsp::lsp_types::Url;

use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings as AnalyzerSettings;
use mago_atom::Atom;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_linter::settings::Settings as LinterSettings;
use mago_orchestrator::service::incremental_analysis::IncrementalAnalysisService;
use mago_prelude::Prelude;
use mago_syntax::settings::ParserSettings;

use crate::consts::PRELUDE_BYTES;
use crate::language_server::ServerConfig;
use crate::language_server::document::OpenDocument;
use crate::language_server::file_analysis;
use crate::language_server::file_analysis::FileAnalysis;
use crate::language_server::linter::LinterContext;
use crate::language_server::workspace::walk_php_files;

#[allow(clippy::large_enum_variant)]
pub enum BackendState {
    Uninitialized,
    Pending(PendingConfig),
    Ready(WorkspaceState),
}

pub struct PendingConfig {
    pub root: PathBuf,
    pub plugin_registry: Arc<PluginRegistry>,
    pub config: Arc<ServerConfig>,
}

pub struct WorkspaceState {
    pub root: PathBuf,
    pub database: Database<'static>,
    pub service: IncrementalAnalysisService,
    pub linter: LinterContext,
    pub config: Arc<ServerConfig>,
    pub open_documents: HashMap<Url, OpenDocument>,
    pub last_diagnostics: HashMap<Url, Vec<Diagnostic>>,
    /// Per-file derived data; lint issues, name index, fold ranges,
    /// AST node spans; all built in one parse + resolve pass per
    /// file content change. See [`super::file_analysis::FileAnalysis`].
    pub file_analyses: HashMap<FileId, (u64, Arc<FileAnalysis>)>,
    /// Per-expression class-name index. Empty when [`ServerConfig::analyzer`]
    /// is `false`.
    pub artifact_cache: HashMap<FileId, (u64, ExpressionTypeIndex)>,
}

#[derive(Debug, Default, Clone)]
pub struct ExpressionTypeIndex {
    pub by_span: HashMap<(u32, u32), Vec<Atom>>,
}

impl WorkspaceState {
    pub fn type_index_for(&mut self, file_id: FileId) -> Option<&ExpressionTypeIndex> {
        if !self.config.analyzer {
            return None;
        }

        let file = self.database.get(&file_id).ok()?;
        let hash = xxhash_rust::xxh3::xxh3_64(file.contents.as_bytes());

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

    pub fn invalidate_artifacts(&mut self, files: &[FileId]) {
        for id in files {
            self.artifact_cache.remove(id);
            self.file_analyses.remove(id);
        }
    }

    /// Return the [`FileAnalysis`] for `file_id`, building it on a cache
    /// miss. Single parse + resolve per content hash; capability handlers
    /// that need lint issues / name resolution / fold ranges / AST node
    /// spans all share this.
    pub fn file_analysis_for(&mut self, file_id: FileId) -> Option<Arc<FileAnalysis>> {
        let file = self.database.get(&file_id).ok()?;
        let hash = xxhash_rust::xxh3::xxh3_64(file.contents.as_bytes());

        if let Some((cached_hash, analysis)) = self.file_analyses.get(&file_id)
            && *cached_hash == hash
        {
            return Some(Arc::clone(analysis));
        }

        let with_semantics = !self.config.analyzer;
        let analysis = Arc::new(file_analysis::build(&file, &self.linter, with_semantics));
        self.file_analyses.insert(file_id, (hash, Arc::clone(&analysis)));
        Some(analysis)
    }

    /// Build (or rebuild) the analysis for every changed file. Called
    /// from `apply_change_atomic` so per-file derived data is fresh by
    /// the time the next capability request comes in.
    pub fn refresh_analyses(&mut self, file_ids: &[FileId]) {
        let with_semantics = !self.config.analyzer;
        for &file_id in file_ids {
            let Ok(file) = self.database.get(&file_id) else {
                self.file_analyses.remove(&file_id);
                continue;
            };
            let hash = xxhash_rust::xxh3::xxh3_64(file.contents.as_bytes());
            let analysis = Arc::new(file_analysis::build(&file, &self.linter, with_semantics));
            self.file_analyses.insert(file_id, (hash, analysis));
        }
    }
}

fn build_index(artifacts: &mago_analyzer::artifacts::AnalysisArtifacts) -> ExpressionTypeIndex {
    use mago_codex::ttype::atomic::TAtomic;
    use mago_codex::ttype::atomic::object::TObject;

    let mut by_span: HashMap<(u32, u32), Vec<Atom>> = HashMap::default();
    for (span, ty) in artifacts.expression_types.iter() {
        let mut classes: Vec<Atom> = Vec::new();
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

pub fn build_workspace(pending: PendingConfig) -> Result<(WorkspaceState, AnalysisResult), String> {
    let PendingConfig { root, plugin_registry, config } = pending;
    let root = root.canonicalize().unwrap_or(root);
    let prelude = Prelude::decode(PRELUDE_BYTES).map_err(|e| format!("decode prelude: {e}"))?;
    let Prelude { mut database, metadata, symbol_references } = prelude;

    let workspace_files = walk_php_files(&root)?;
    for file in workspace_files {
        database.add(file);
    }

    let parser_settings = ParserSettings::default();
    let mut service = IncrementalAnalysisService::new(
        database.read_only(),
        metadata,
        symbol_references,
        AnalyzerSettings::default(),
        parser_settings,
        plugin_registry,
    );

    let linter = LinterContext::new(LinterSettings::default(), parser_settings);

    let analysis_result = if config.analyzer {
        service.analyze().map_err(|err| format!("initial analyze: {err}"))?
    } else {
        AnalysisResult::new(mago_codex::reference::SymbolReferences::new())
    };

    let workspace = WorkspaceState {
        root,
        database,
        service,
        linter,
        config,
        open_documents: HashMap::default(),
        last_diagnostics: HashMap::default(),
        file_analyses: HashMap::default(),
        artifact_cache: HashMap::default(),
    };

    Ok((workspace, analysis_result))
}
