//! Backend state lifecycle.
//!
//! On startup the backend is [`BackendState::Uninitialized`]. The
//! `initialize` request triggers a one-shot bootstrap that walks
//! the workspace, builds the database, runs an initial full analysis (if
//! enabled by the [`super::ServerConfig`]), and transitions to
//! [`BackendState::Ready`].
//!
//! The analysis state itself; the database, the analysis service, and the
//! per-file caches; lives in a transport-agnostic [`mago_server::Server`].
//! [`WorkspaceState`] wraps one such server with the LSP-specific bookkeeping
//! (open buffers, last-published diagnostics) and a few thin delegating
//! accessors so the capability handlers keep stable call sites.

use std::borrow::Cow;
use std::cmp::Reverse;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use clap::ColorChoice;
use foldhash::HashMap;
use tower_lsp_server::ls_types::Diagnostic;
use tower_lsp_server::ls_types::Uri;

use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::plugin::create_registry_with_plugins;
use mago_database::Database;
use mago_database::DatabaseConfiguration;
use mago_database::exclusion::Exclusion;
use mago_database::file::FileId;
use mago_database::loader::DatabaseLoader;
use mago_database::membership::WorkspaceMatcher;
use mago_linter::integration::IntegrationSet;
use mago_linter::settings::Settings as LinterSettings;
use mago_prelude::Prelude;
use mago_server::Features;
use mago_server::FileAnalysis;
use mago_server::Server;
use mago_server::Settings as ServerSettings;

use crate::config::Configuration;
use crate::consts::PRELUDE_BYTES;
use crate::language_server::ServerConfig;
use crate::language_server::document::OpenDocument;

#[allow(clippy::large_enum_variant)]
pub enum BackendState {
    Uninitialized,
    Ready(WorkspaceRegistry),
}

/// All open workspaces, one [`WorkspaceState`] per editor workspace folder.
///
/// Requests are routed to the workspace whose root is the longest path prefix
/// of the target file, so nested folders resolve to the most specific project.
pub struct WorkspaceRegistry {
    workspaces: Vec<WorkspaceState>,
}

impl WorkspaceRegistry {
    /// Build a registry from per-folder workspaces, ordered so longest-prefix
    /// routing picks the most specific root first.
    #[must_use]
    pub fn new(mut workspaces: Vec<WorkspaceState>) -> Self {
        sort_by_root_len(&mut workspaces);
        Self { workspaces }
    }

    /// Add a workspace, keeping longest-prefix routing order.
    pub fn add(&mut self, workspace: WorkspaceState) {
        self.workspaces.push(workspace);
        sort_by_root_len(&mut self.workspaces);
    }

    /// Remove and return the workspace rooted exactly at `root`, if present.
    pub fn remove(&mut self, root: &Path) -> Option<WorkspaceState> {
        let index = self.workspaces.iter().position(|ws| ws.root == root)?;
        Some(self.workspaces.remove(index))
    }

    /// Whether a workspace is already rooted exactly at `root`.
    #[must_use]
    pub fn contains_root(&self, root: &Path) -> bool {
        self.workspaces.iter().any(|ws| ws.root == root)
    }

    /// The workspace owning `path` (longest root prefix), if any.
    #[must_use]
    pub fn for_path(&self, path: &Path) -> Option<&WorkspaceState> {
        self.workspaces.iter().find(|ws| path.starts_with(&ws.root))
    }

    /// The workspace owning `path` (longest root prefix), mutably.
    pub fn for_path_mut(&mut self, path: &Path) -> Option<&mut WorkspaceState> {
        self.workspaces.iter_mut().find(|ws| path.starts_with(&ws.root))
    }

    /// Whether any workspace's source matcher accepts `path`.
    #[must_use]
    pub fn tracks(&self, path: &Path) -> bool {
        self.workspaces.iter().any(|ws| ws.matcher.contains(path))
    }

    /// Iterate every workspace (e.g. for cross-workspace symbol search).
    pub fn iter(&self) -> impl Iterator<Item = &WorkspaceState> + '_ {
        self.workspaces.iter()
    }
}

/// Order workspaces by descending root-path length so longest-prefix routing
/// resolves a path to its most specific (deepest) workspace.
fn sort_by_root_len(workspaces: &mut [WorkspaceState]) {
    workspaces.sort_by_key(|b| Reverse(b.root.as_os_str().len()));
}

pub struct WorkspaceState {
    pub root: PathBuf,
    pub matcher: WorkspaceMatcher,
    pub server: Server,
    /// This workspace's own configuration, discovered from `{root}/mago.*` on
    /// `initialize` (or inherited from the launch config when the folder has
    /// none). Drives ignore patterns, globs, and formatter settings.
    pub configuration: Configuration,
    /// Global feature switches (from the CLI flags), shared by all workspaces.
    pub features: Features,
    pub open_documents: HashMap<Uri, OpenDocument>,
    pub last_diagnostics: HashMap<Uri, Vec<Diagnostic>>,
}

impl WorkspaceState {
    /// Drop cached derived data for the given files.
    pub fn invalidate_artifacts(&mut self, files: &[FileId]) {
        self.server.invalidate_artifacts(files);
    }

    /// Return the [`FileAnalysis`] for `file_id`, building it on a cache miss.
    pub fn file_analysis_for(&mut self, file_id: FileId) -> Option<Arc<FileAnalysis>> {
        self.server.file_analysis_for(file_id)
    }

    /// Build (or rebuild) the analysis for every changed file.
    pub fn refresh_analyses(&mut self, file_ids: &[FileId]) {
        self.server.refresh_analyses(file_ids);
    }

    /// Borrow the workspace file database.
    #[must_use]
    pub fn database(&self) -> &Database<'static> {
        self.server.database()
    }

    /// Mutably borrow the workspace file database.
    pub fn database_mut(&mut self) -> &mut Database<'static> {
        self.server.database_mut()
    }
}

pub fn build_workspace(
    root: PathBuf,
    server_config: Arc<ServerConfig>,
) -> Result<(WorkspaceState, AnalysisResult), String> {
    let root = root.canonicalize().unwrap_or(root);
    let prelude = Prelude::decode(PRELUDE_BYTES).map_err(|e| format!("decode prelude: {e}"))?;
    let Prelude { database: prelude_db, metadata, symbol_references } = prelude;

    let configuration = resolve_workspace_configuration(&root, &server_config);

    let parser_settings = configuration.parser.to_settings();
    let analyzer_settings = configuration.analyzer.to_settings(configuration.php_version, ColorChoice::Never, false);
    let glob = configuration.source.glob.to_database_settings();
    let linter_settings = LinterSettings {
        php_version: configuration.php_version,
        integrations: IntegrationSet::from_slice(&configuration.linter.integrations),
        rules: configuration.linter.rules.clone(),
        glob,
    };

    let db_configuration = build_database_configuration(&root, &configuration);
    let matcher = WorkspaceMatcher::from_configuration(&db_configuration)
        .map_err(|err| format!("compile source matcher: {err}"))?;
    let database = DatabaseLoader::new(db_configuration)
        .with_database(prelude_db)
        .load()
        .map_err(|err| format!("load database: {err}"))?;

    let features =
        Features { analyzer: server_config.analyzer, linter: server_config.linter, formatter: server_config.formatter };
    let plugin_registry = Arc::new(create_registry_with_plugins(
        &configuration.analyzer.plugins,
        configuration.analyzer.disable_default_plugins,
    ));

    let settings = ServerSettings {
        php_version: configuration.php_version,
        features,
        parser: parser_settings,
        analyzer: analyzer_settings,
        linter: linter_settings,
        formatter: configuration.formatter.settings,
        plugin_registry,
    };

    let mut server = Server::new(database, metadata, symbol_references, settings);
    let analysis_result = server.analyze().map_err(|err| format!("initial analyze: {err}"))?;

    let workspace = WorkspaceState {
        root,
        matcher,
        server,
        configuration,
        features,
        open_documents: HashMap::default(),
        last_diagnostics: HashMap::default(),
    };

    Ok((workspace, analysis_result))
}

/// Resolve the configuration for a workspace folder.
///
/// If the folder has its own `mago.{toml,json,yaml,yml}`, that file is loaded
/// directly (no global `~/.config` fallback search, so the result depends only
/// on the workspace). Otherwise the launch-time configuration is inherited.
fn resolve_workspace_configuration(root: &Path, server_config: &ServerConfig) -> Configuration {
    let Some(file) = find_workspace_config_file(root) else {
        return server_config.configuration.clone();
    };

    match Configuration::load(
        Some(root.to_path_buf()),
        Some(&file),
        None,
        None,
        server_config.configuration.allow_unsupported_php_version,
        true,
    ) {
        Ok(configuration) => configuration,
        Err(err) => {
            tracing::warn!(
                root = %root.display(),
                error = %err,
                "failed to load workspace configuration; inheriting launch defaults",
            );
            server_config.configuration.clone()
        }
    }
}

/// First `mago.{toml,json,yaml,yml}` directly under `root`, if any.
fn find_workspace_config_file(root: &Path) -> Option<PathBuf> {
    ["mago.toml", "mago.json", "mago.yaml", "mago.yml"].into_iter().map(|name| root.join(name)).find(|p| p.is_file())
}

/// Build the [`DatabaseConfiguration`] the LSP uses to scan the workspace,
/// honouring `[source].paths`, `includes`, `excludes`, `extensions`, and the
/// glob matcher options exactly as `mago lint` / `mago analyze` would.
fn build_database_configuration(root: &Path, configuration: &Configuration) -> DatabaseConfiguration<'static> {
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

    DatabaseConfiguration { workspace, paths, includes, excludes, extensions, glob: source.glob.to_database_settings() }
}
