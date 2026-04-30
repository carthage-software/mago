//! `tower-lsp` glue: locks state, dispatches each LSP request to the
//! matching pure handler in [`super::capabilities`]. Lifecycle and
//! workspace-mutation helpers live in [`sync`]; the trait impl below
//! contains only thin forwarders.

use std::sync::Arc;
use std::sync::Mutex;

use tower_lsp::Client;
use tower_lsp::LanguageServer;
use tower_lsp::jsonrpc::Result as JsonRpcResult;
use tower_lsp::lsp_types::*;

use mago_analyzer::plugin::PluginRegistry;

use crate::language_server::ServerConfig;
use crate::language_server::capabilities;
use crate::language_server::capabilities::server_capabilities;
use crate::language_server::position::offset_at_position;
use crate::language_server::state::BackendState;
use crate::language_server::state::PendingConfig;
use crate::language_server::workspace::workspace_root;

mod sync;

use sync::file_for_uri;
use sync::traced;

pub struct Backend {
    client: Client,
    config: Arc<ServerConfig>,
    state: Arc<Mutex<BackendState>>,
}

impl Backend {
    #[must_use]
    pub fn new(client: Client, config: Arc<ServerConfig>) -> Self {
        Self { client, config, state: Arc::new(Mutex::new(BackendState::Uninitialized)) }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> JsonRpcResult<InitializeResult> {
        if let Some(root) = workspace_root(&params) {
            self.client
                .log_message(MessageType::INFO, format!("mago-server: workspace root = {}", root.display()))
                .await;

            let plugin_registry = Arc::new(PluginRegistry::with_library_providers());
            *self.state.lock().unwrap() =
                BackendState::Pending(PendingConfig { root, plugin_registry, config: Arc::clone(&self.config) });
        } else {
            self.client.log_message(MessageType::WARNING, "mago-server: no workspace root provided").await;
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(&self.config),
            server_info: Some(ServerInfo {
                name: "mago-server".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        let client = self.client.clone();
        tokio::spawn(async move {
            let registration = Registration {
                id: "watch-php-files".into(),
                method: "workspace/didChangeWatchedFiles".into(),
                register_options: Some(
                    serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                        watchers: vec![FileSystemWatcher {
                            glob_pattern: GlobPattern::String("**/*.php".into()),
                            kind: None,
                        }],
                    })
                    .unwrap(),
                ),
            };
            if let Err(err) = client.register_capability(vec![registration]).await {
                client.log_message(MessageType::ERROR, format!("mago-server: register watcher: {err}")).await;
            }
        });

        self.bootstrap().await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let TextDocumentItem { uri, text, version, .. } = params.text_document;
        self.apply_buffer_open(uri, text, version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let Some(change) = params.content_changes.into_iter().next_back() else {
            return;
        };
        self.apply_buffer_change(uri, change.text, params.text_document.version).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.apply_buffer_close(params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.apply_disk_change(params.text_document.uri).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        for change in params.changes {
            match change.typ {
                FileChangeType::CREATED | FileChangeType::CHANGED => {
                    self.apply_disk_change(change.uri).await;
                }
                FileChangeType::DELETED => {
                    self.apply_disk_delete(change.uri).await;
                }
                _ => {}
            }
        }
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> JsonRpcResult<Option<Vec<FoldingRange>>> {
        traced("folding_range", || {
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let file_id = file.id;
                let analysis = ws.file_analysis_for(file_id)?;
                Some(capabilities::folding_range::compute(&analysis))
            });
            Ok(result.flatten())
        })
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> JsonRpcResult<Option<Vec<TextEdit>>> {
        traced("formatting", || {
            if !self.config.formatter {
                return Ok(None);
            }
            Ok(self
                .with_file(&params.text_document.uri, |file, _| capabilities::formatting::compute(file))
                .flatten()
                .map(|edit| vec![edit]))
        })
    }

    async fn hover(&self, params: HoverParams) -> JsonRpcResult<Option<Hover>> {
        traced("hover", || {
            let TextDocumentPositionParams { text_document, position } = params.text_document_position_params;
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                let analysis = ws.file_analysis_for(file.id)?;
                capabilities::hover::compute(ws.service.codebase(), analysis.resolved(), &file, offset)
            });
            Ok(result.flatten())
        })
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> JsonRpcResult<Option<GotoDefinitionResponse>> {
        traced("goto_definition", || {
            let TextDocumentPositionParams { text_document, position } = params.text_document_position_params;
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                let analysis = ws.file_analysis_for(file.id)?;
                capabilities::definition::compute(&ws.database, ws.service.codebase(), analysis.resolved(), offset)
                    .map(GotoDefinitionResponse::Scalar)
            });
            Ok(result.flatten())
        })
    }

    async fn selection_range(&self, params: SelectionRangeParams) -> JsonRpcResult<Option<Vec<SelectionRange>>> {
        traced("selection_range", || {
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let file_id = file.id;
                let analysis = ws.file_analysis_for(file_id)?;
                let offsets: Vec<u32> = params.positions.iter().map(|p| offset_at_position(&file, *p)).collect();
                Some(capabilities::selection_range::compute(&analysis, &file, &offsets))
            });
            Ok(result.flatten())
        })
    }

    async fn code_action(&self, params: CodeActionParams) -> JsonRpcResult<Option<CodeActionResponse>> {
        traced("code_action", || {
            let result = self.with_workspace(|ws| Some(capabilities::code_action::compute(ws, &params)));
            Ok(result.flatten())
        })
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> JsonRpcResult<Option<SemanticTokensResult>> {
        traced("semantic_tokens_full", || {
            Ok(self.with_file(&params.text_document.uri, |file, _| {
                let data = capabilities::semantic_tokens::compute(file);
                SemanticTokensResult::Tokens(SemanticTokens { result_id: None, data })
            }))
        })
    }

    async fn references(&self, params: ReferenceParams) -> JsonRpcResult<Option<Vec<Location>>> {
        traced("references", || {
            let TextDocumentPositionParams { text_document, position } = params.text_document_position;
            let include_decl = params.context.include_declaration;
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                Some(capabilities::references::compute(ws, &file, offset, include_decl))
            });
            Ok(result.flatten())
        })
    }

    async fn symbol(&self, params: WorkspaceSymbolParams) -> JsonRpcResult<Option<Vec<SymbolInformation>>> {
        traced("workspace_symbol", || {
            Ok(self.with_workspace(|ws| {
                capabilities::workspace_symbol::compute(&ws.database, ws.service.codebase(), &params.query)
            }))
        })
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> JsonRpcResult<Option<SignatureHelp>> {
        traced("signature_help", || {
            if !self.config.analyzer {
                return Ok(None);
            }
            let TextDocumentPositionParams { text_document, position } = params.text_document_position_params;
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                let analysis = ws.file_analysis_for(file.id)?;
                capabilities::signature_help::compute(ws.service.codebase(), analysis.resolved(), &file, offset)
            });
            Ok(result.flatten())
        })
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> JsonRpcResult<Option<Vec<InlayHint>>> {
        traced("inlay_hint", || {
            if !self.config.analyzer {
                return Ok(None);
            }
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let start = offset_at_position(&file, params.range.start);
                let end = offset_at_position(&file, params.range.end);
                let analysis = ws.file_analysis_for(file.id)?;
                Some(capabilities::inlay_hint::compute(ws.service.codebase(), analysis.resolved(), &file, start, end))
            });
            Ok(result.flatten())
        })
    }

    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> JsonRpcResult<Option<PrepareRenameResponse>> {
        traced("prepare_rename", || {
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let offset = offset_at_position(&file, params.position);
                let analysis = ws.file_analysis_for(file.id)?;
                capabilities::rename::prepare(analysis.resolved(), &file, offset)
            });
            Ok(result.flatten())
        })
    }

    async fn rename(&self, params: RenameParams) -> JsonRpcResult<Option<WorkspaceEdit>> {
        traced("rename", || {
            let TextDocumentPositionParams { text_document, position } = params.text_document_position;
            let new_name = params.new_name;
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                capabilities::rename::compute(ws, &file, offset, new_name)
            });
            Ok(result.flatten())
        })
    }

    async fn document_link(&self, params: DocumentLinkParams) -> JsonRpcResult<Option<Vec<DocumentLink>>> {
        traced("document_link", || {
            let result = self.with_workspace(|ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                Some(capabilities::document_link::compute(&ws.database, ws.service.codebase(), &file))
            });
            Ok(result.flatten())
        })
    }

    async fn code_lens(&self, params: CodeLensParams) -> JsonRpcResult<Option<Vec<CodeLens>>> {
        traced("code_lens", || {
            if !self.config.analyzer {
                return Ok(None);
            }
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let id = file.id;
                Some(capabilities::code_lens::compute(ws, &file, id))
            });
            Ok(result.flatten())
        })
    }

    async fn completion(&self, params: CompletionParams) -> JsonRpcResult<Option<CompletionResponse>> {
        traced("completion", || {
            let TextDocumentPositionParams { text_document, position } = params.text_document_position;
            let result = self.with_workspace_mut(|ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                let file_id = file.id;
                let type_index = ws.type_index_for(file_id).cloned();
                Some(capabilities::completion::compute(
                    &ws.database,
                    ws.service.codebase(),
                    type_index.as_ref(),
                    &file,
                    offset,
                ))
            });
            Ok(result.flatten())
        })
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }
}
