//! Load workspace configuration, source files, and initial analysis.

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use clap::ColorChoice;

use mago_analyzer::plugin::create_registry_with_plugins;
use mago_database::DatabaseConfiguration;
use mago_database::exclusion::Exclusion;
use mago_database::loader::DatabaseLoader;
use mago_database::membership::WorkspaceMatcher;
use mago_linter::integration::IntegrationSet;
use mago_linter::settings::Settings as LinterSettings;
use mago_prelude::Prelude;
use mago_reporting::CompiledIgnoreSet;
use mago_server::Features;
use mago_server::Server;
use mago_server::Settings as ServerSettings;
use mago_server::workspace::Workspace;
use mago_server::workspace::directory_aliases::DirectoryAliases;

use crate::config::Configuration;
use crate::consts::PRELUDE_BYTES;
use crate::language_server::ServerConfig;
use crate::language_server::diagnostic_cache::DiagnosticCache;
use crate::language_server::sources::SourceFiles;
use crate::language_server::state::WorkspaceState;

pub fn build_workspace(folder: PathBuf, server_config: Arc<ServerConfig>) -> Result<WorkspaceState, String> {
    let root = folder.canonicalize().unwrap_or_else(|_| folder.clone());
    let root = SourceFiles::workspace_for_cached_path(&root)?.unwrap_or(root);
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
    let (database, directory_symlinks) = DatabaseLoader::new(db_configuration)
        .with_database(prelude_db)
        .load_with_directory_symlinks()
        .map_err(|err| format!("load database: {err}"))?;
    let directory_aliases = DirectoryAliases::new(directory_symlinks, &matcher);

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
    analyze_workspace(&mut server, features)?;
    let analyzer_ignores = CompiledIgnoreSet::compile(&configuration.analyzer.ignore, glob);

    let sources = SourceFiles::new(&root);
    let mut core = Workspace::new(folder, root, server, matcher, analyzer_ignores);
    core.directory_aliases = directory_aliases;
    core.set_source_root(sources.root().to_path_buf());
    let workspace = WorkspaceState { core, sources, configuration, diagnostics: DiagnosticCache::default() };

    Ok(workspace)
}

/// Finish all initial analysis before the workspace reaches the async backend.
fn analyze_workspace(server: &mut Server, features: Features) -> Result<(), String> {
    let started = Instant::now();
    server.analyze_without_collecting().map_err(|err| format!("initial analyze: {err}"))?;
    tracing::info!(
        elapsed = ?started.elapsed(),
        issues = server.analyzer_issues().map(|issues| issues.len()).sum::<usize>(),
        "analyzer pass complete",
    );
    if features.linter || !features.analyzer {
        let started = Instant::now();
        server.refresh_all_host_analyses();
        tracing::info!(
            elapsed = ?started.elapsed(),
            files = server.analyses().count(),
            issues = server.lint_issues().map(|issues| issues.len()).sum::<usize>(),
            "file analysis pass complete",
        );
    }
    Ok(())
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

    DatabaseConfiguration {
        workspace,
        paths,
        includes,
        excludes,
        extensions,
        patches: source.patches.iter().cloned().map(|path| Cow::Owned(path.into_bytes())).collect(),
        glob: source.glob.to_database_settings(),
    }
}
