use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tower_lsp_server::gen_lsp_types::LspErrorCodes;
use tower_lsp_server::gen_lsp_types::{FileChangeType, FileEvent, Uri, WorkspaceSymbolParams};
use tower_lsp_server::jsonrpc::ErrorCode;
use tower_lsp_server::{LanguageServer, LspService};

use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::plugin::context::HookContext;
use mago_analyzer::plugin::error::PluginError;
use mago_analyzer::plugin::hook::{HookAction, HookResult, ProgramHook};
use mago_analyzer::plugin::provider::{Provider as HookProvider, ProviderMeta};
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::file::{File, FileType};
use mago_database::membership::WorkspaceMatcher;
use mago_database::{Database, DatabaseConfiguration};
use mago_reporting::CompiledIgnoreSet;
use mago_server::{Features, Server, Settings};
use mago_syntax::cst::Program;

use super::*;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;
use crate::language_server::diagnostic_cache::DiagnosticCache;
use crate::language_server::sources::SourceFiles;
use crate::language_server::state::WorkspaceState;
use crate::language_server::uri::UriFilePath;

struct FailingHook(Arc<AtomicBool>);

impl HookProvider for FailingHook {
    fn meta() -> &'static ProviderMeta {
        static META: ProviderMeta = ProviderMeta::new("test-failure", "Test failure", "Fail on request");
        &META
    }
}

impl ProgramHook for FailingHook {
    fn before_program(&self, _: &File, _: &Program<'_>, _: &mut HookContext<'_, '_>) -> HookResult<HookAction> {
        if self.0.load(Ordering::Relaxed) {
            Err(PluginError::Internal { reason: "test failure".into() })
        } else {
            Ok(HookAction::Continue)
        }
    }
}

#[tokio::test]
async fn failed_analysis_clears_diagnostics_and_retries_unchanged_save() {
    for linter in [false, true] {
        for empty_hint in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            let configuration = DatabaseConfiguration::new(&root, vec![], vec![], vec![], vec![]);
            let matcher = WorkspaceMatcher::from_configuration(&configuration).unwrap();
            let mut database = Database::new(configuration.into_static());
            let mut files = Vec::new();
            for (name, source) in [
                ("old.php", "<?php function wrong(): int { return 'wrong'; }"),
                ("changed.php", "<?php class Before {}"),
                ("kept.php", "<?php eval('');"),
                ("deleted.php", "<?php function gone(): int { return 'wrong'; }"),
            ] {
                let path = root.join(name);
                std::fs::write(&path, source).unwrap();
                files.push(database.add(File::new(
                    name.as_bytes().to_vec().into(),
                    FileType::Host,
                    Some(path),
                    source.as_bytes().into(),
                )));
            }
            let fail = Arc::new(AtomicBool::new(false));
            let mut plugins = PluginRegistry::new();
            plugins.register_program_hook(FailingHook(Arc::clone(&fail)));
            let mut server = Server::new(
                database,
                CodebaseMetadata::new(),
                SymbolReferences::new(),
                Settings {
                    php_version: Default::default(),
                    features: Features { analyzer: true, linter, formatter: true },
                    parser: Default::default(),
                    analyzer: Default::default(),
                    linter: Default::default(),
                    formatter: Default::default(),
                    plugin_registry: Arc::new(plugins),
                },
            );
            server.analyze_without_collecting().unwrap();
            server.refresh_all_host_analyses();
            let kept = server.file_analysis_for(files[2]).unwrap();
            let previous = server.file_analysis_for(files[1]).unwrap();
            let workspace = WorkspaceState {
                sources: SourceFiles::new(&root),
                core: mago_server::workspace::Workspace::new(
                    root.clone(),
                    root.clone(),
                    server,
                    matcher,
                    CompiledIgnoreSet::compile(&[], Default::default()),
                ),
                configuration: crate::config::Configuration::from_workspace(root.clone()),
                diagnostics: DiagnosticCache::default(),
            };
            let mut registry = WorkspaceRegistry::new(vec![workspace]);
            let old_uri = Uri::from_file_path(root.join("old.php")).unwrap();
            let changed_uri = Uri::from_file_path(root.join("changed.php")).unwrap();
            let deleted_uri = Uri::from_file_path(root.join("deleted.php")).unwrap();
            assert!(
                registry
                    .publication
                    .take_initial()
                    .iter()
                    .any(|(uri, payload)| { *uri == old_uri && !payload.diagnostics.is_empty() })
            );
            fail.store(true, Ordering::Relaxed);
            let events = if empty_hint {
                assert!(registry.for_path_mut(&root).unwrap().core.server.analyze_without_collecting().is_err());
                vec![FileEvent { uri: changed_uri.clone(), kind: FileChangeType::Changed }]
            } else {
                std::fs::write(
                    root.join("changed.php"),
                    "<?php class After { public function run(): void { eval(''); } }",
                )
                .unwrap();
                std::fs::remove_file(root.join("deleted.php")).unwrap();
                vec![
                    FileEvent { uri: changed_uri.clone(), kind: FileChangeType::Changed },
                    FileEvent { uri: deleted_uri.clone(), kind: FileChangeType::Deleted },
                ]
            };
            let update = registry.change_disk_files(events);
            assert_eq!(update.errors.len(), 1);
            assert_eq!(update.changed_files, if empty_hint { 0 } else { 2 });
            assert!(update.refresh_code_lenses);
            if !linter {
                assert!(
                    update.diagnostics.iter().any(|(uri, payload)| *uri == old_uri && payload.diagnostics.is_empty())
                );
            }
            let workspace = registry.for_path_mut(&root).unwrap();
            assert!(!workspace.core.server.is_initialized());
            assert!(workspace.core.server.shared_analyzer_issues().all(|(_, _, issues)| issues.is_empty()));
            assert!(Arc::ptr_eq(&kept, &workspace.core.server.file_analysis_for(files[2]).unwrap()));
            if !empty_hint {
                assert!(!Arc::ptr_eq(&previous, &workspace.core.server.file_analysis_for(files[1]).unwrap()));
                assert!(workspace.core.server.lint_issue_sources().all(|(file, _)| file != files[3]));
                if linter {
                    assert!(!workspace.core.server.file_analysis_for(files[1]).unwrap().lint_issues.is_empty());
                }
                assert!(
                    update
                        .diagnostics
                        .iter()
                        .any(|(uri, payload)| *uri == deleted_uri && payload.diagnostics.is_empty())
                );
            }
            let (service, socket) = LspService::new(|client| Backend::new(client, Arc::new(ServerConfig::default())));
            drop(socket);
            let backend = service.inner();
            let capabilities = ClientPreferences::default().server_capabilities(&backend.config);
            assert!(backend.session.set(Session::new(ClientPreferences::default(), registry, capabilities)).is_ok());
            for provider in [Provider::Hover, Provider::CallHierarchy, Provider::TypeHierarchy, Provider::CodeAction] {
                let error = backend.workspace_query(provider, &changed_uri, |_| Some(())).unwrap_err();
                assert_eq!(error.code, ErrorCode::ServerError(i32::from(LspErrorCodes::RequestFailed).into()));
            }
            assert!(backend.query(Provider::DocumentSymbol, &changed_uri, |_| Some(())).unwrap().is_some());
            assert_eq!(
                backend.symbol(WorkspaceSymbolParams::default()).await.unwrap_err().code,
                ErrorCode::ServerError(i32::from(LspErrorCodes::RequestFailed).into())
            );

            fail.store(false, Ordering::Relaxed);
            let update = backend
                .with_registry_mut(|registry| {
                    registry
                        .change_disk_files(vec![FileEvent { uri: changed_uri.clone(), kind: FileChangeType::Changed }])
                })
                .unwrap();
            assert!(update.errors.is_empty(), "{:?}", update.errors);
            assert_eq!(update.changed_files, 0);
            assert!(update.refresh_code_lenses);
            assert!(backend.query(Provider::Hover, &changed_uri, |_| Some(())).unwrap().is_some());
            assert!(backend.symbol(WorkspaceSymbolParams::default()).await.is_ok());
            assert!(update.diagnostics.iter().any(|(uri, payload)| *uri == old_uri && !payload.diagnostics.is_empty()));
        }
    }
}
