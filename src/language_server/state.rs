//! Backend state lifecycle.
//!
//! On startup the backend is [`BackendState::Uninitialized`]. The
//! `initialize` request triggers a one-shot bootstrap that walks
//! the workspace, builds the database, runs an initial full analysis (if
//! enabled by the [`super::ServerConfig`]), and transitions to
//! [`BackendState::Ready`].

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use clap::ColorChoice;
use foldhash::HashMap;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use tower_lsp_server::ls_types::Diagnostic;
use tower_lsp_server::ls_types::Uri;

use mago_analyzer::analysis_result::AnalysisResult;
use mago_database::Database;
use mago_database::DatabaseConfiguration;
use mago_database::DatabaseReader;
use mago_database::exclusion::Exclusion;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_database::loader::DatabaseLoader;
use mago_linter::integration::IntegrationSet;
use mago_linter::settings::Settings as LinterSettings;
use mago_orchestrator::service::incremental_analysis::IncrementalAnalysisService;
use mago_prelude::Prelude;
use mago_word::Word;
use xxhash_rust::xxh3;

use crate::config::Configuration;
use crate::consts::PRELUDE_BYTES;
use crate::language_server::ServerConfig;
use crate::language_server::document::OpenDocument;
use crate::language_server::file_analysis;
use crate::language_server::file_analysis::FileAnalysis;
use crate::language_server::linter::LinterContext;

#[allow(clippy::large_enum_variant)]
pub enum BackendState {
    Uninitialized,
    Ready(WorkspaceState),
}

pub struct WorkspaceState {
    pub root: PathBuf,
    pub database: Database<'static>,
    pub service: IncrementalAnalysisService,
    pub linter: LinterContext,
    pub config: Arc<ServerConfig>,
    pub open_documents: HashMap<Uri, OpenDocument>,
    pub last_diagnostics: HashMap<Uri, Vec<Diagnostic>>,
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
    pub by_span: HashMap<(u32, u32), Vec<Word>>,
}

impl WorkspaceState {
    pub fn type_index_for(&mut self, file_id: FileId) -> Option<&ExpressionTypeIndex> {
        if !self.config.analyzer {
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
        let hash = xxh3::xxh3_64(&file.contents);

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
            if file.file_type != FileType::Host {
                continue;
            }
            let hash = xxh3::xxh3_64(&file.contents);
            let analysis = Arc::new(file_analysis::build(&file, &self.linter, with_semantics));
            self.file_analyses.insert(file_id, (hash, analysis));
        }
    }
}

fn build_index(artifacts: &mago_analyzer::artifacts::AnalysisArtifacts) -> ExpressionTypeIndex {
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

pub fn build_workspace(root: PathBuf, config: Arc<ServerConfig>) -> Result<(WorkspaceState, AnalysisResult), String> {
    let root = root.canonicalize().unwrap_or(root);
    let prelude = Prelude::decode(PRELUDE_BYTES).map_err(|e| format!("decode prelude: {e}"))?;
    let Prelude { database: prelude_db, metadata, symbol_references } = prelude;

    let configuration = &config.configuration;
    let parser_settings = configuration.parser.to_settings();
    let analyzer_settings = configuration.analyzer.to_settings(configuration.php_version, ColorChoice::Never, false);
    let glob = configuration.source.glob.to_database_settings();
    let linter_settings = LinterSettings {
        php_version: configuration.php_version,
        integrations: IntegrationSet::from_slice(&configuration.linter.integrations),
        rules: configuration.linter.rules.clone(),
        glob,
    };

    let database = load_workspace_database(&root, configuration, prelude_db)?;

    let mut service = IncrementalAnalysisService::new(
        database.read_only(),
        metadata,
        symbol_references,
        analyzer_settings,
        parser_settings,
        Arc::clone(&config.plugin_registry),
    );

    let linter = LinterContext::new(linter_settings, parser_settings);

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

/// Load the workspace database the same way `mago lint` / `mago analyze`
/// would: honour `[source].paths`, `includes`, `excludes`, `extensions`,
/// and the glob matcher options. The prelude database is merged in as
/// the analysis baseline.
fn load_workspace_database(
    root: &Path,
    configuration: &Configuration,
    prelude_db: Database<'static>,
) -> Result<Database<'static>, String> {
    let source = &configuration.source;
    let workspace = Cow::<'static, Path>::Owned(root.to_path_buf());

    // No `[source].paths`: fall back to scanning the workspace root, the
    // same default behaviour `mago lint` / `mago analyze` rely on.
    let paths: Vec<Cow<'static, [u8]>> = if source.paths.is_empty() {
        vec![Cow::Owned(root.to_string_lossy().into_owned().into_bytes())]
    } else {
        source.paths.iter().cloned().map(|s| Cow::<'static, [u8]>::Owned(s.into_bytes())).collect()
    };

    let includes: Vec<Cow<'static, [u8]>> =
        source.includes.iter().cloned().map(|s| Cow::<'static, [u8]>::Owned(s.into_bytes())).collect();
    let extensions: Vec<Cow<'static, [u8]>> = if source.extensions.is_empty() {
        vec![Cow::Borrowed(b"php")]
    } else {
        source.extensions.iter().cloned().map(|s| Cow::<'static, [u8]>::Owned(s.into_bytes())).collect()
    };

    let excludes: Vec<Exclusion<'static>> = source
        .excludes
        .iter()
        .map(|pattern| {
            if pattern.contains('*') {
                if let Some(stripped) = pattern.strip_prefix("./") {
                    Exclusion::Pattern(Cow::Owned(root.join(stripped).to_string_lossy().into_owned()))
                } else {
                    Exclusion::Pattern(Cow::Owned(pattern.clone()))
                }
            } else {
                let path = std::path::PathBuf::from(pattern);
                let path = if path.is_absolute() { path } else { root.join(path) };
                Exclusion::Path(Cow::Owned(path.canonicalize().unwrap_or(path)))
            }
        })
        .collect();

    let db_configuration = DatabaseConfiguration {
        workspace,
        paths,
        includes,
        excludes,
        extensions,
        glob: source.glob.to_database_settings(),
    };

    DatabaseLoader::new(db_configuration)
        .with_database(prelude_db)
        .load()
        .map_err(|err| format!("load database: {err}"))
}
