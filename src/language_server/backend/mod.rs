//! `tower-lsp` glue: locks state, dispatches each LSP request to the
//! matching pure handler in [`super::capabilities`]. Lifecycle and
//! workspace-mutation helpers live in [`sync`]; the trait impl below
//! contains only thin forwarders.

use std::sync::Arc;
use std::sync::Mutex;

use tower_lsp_server::Client;
use tower_lsp_server::LanguageServer;
use tower_lsp_server::jsonrpc::Result as JsonRpcResult;
use tower_lsp_server::ls_types::*;

use crate::language_server::ServerConfig;
use crate::language_server::capabilities::code_action;
use crate::language_server::capabilities::code_lens;
use crate::language_server::capabilities::completion;
use crate::language_server::capabilities::definition;
use crate::language_server::capabilities::document_link;
use crate::language_server::capabilities::folding_range;
use crate::language_server::capabilities::formatting;
use crate::language_server::capabilities::hover;
use crate::language_server::capabilities::inlay_hint;
use crate::language_server::capabilities::references;
use crate::language_server::capabilities::rename;
use crate::language_server::capabilities::selection_range;
use crate::language_server::capabilities::semantic_tokens;
use crate::language_server::capabilities::server_capabilities;
use crate::language_server::capabilities::signature_help;
use crate::language_server::capabilities::workspace_symbol;
use crate::language_server::position::offset_at_position;
use crate::language_server::state::BackendState;
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

    fn tracks(&self, uri: &Uri) -> bool {
        let Some(path) = uri.to_file_path() else {
            return false;
        };

        self.with_workspace(|workspace| workspace.matcher.contains(path.as_ref())).unwrap_or(true)
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> JsonRpcResult<InitializeResult> {
        let root = self.config.workspace_override.clone().or_else(|| workspace_root(&params));
        if let Some(root) = root {
            self.client
                .log_message(MessageType::INFO, format!("mago-server: workspace root = {}", root.display()))
                .await;

            self.bootstrap(root).await;
        } else {
            self.client.log_message(MessageType::WARNING, "mago-server: no workspace root provided").await;
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(&self.config),
            server_info: Some(ServerInfo {
                name: "mago-server".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            offset_encoding: None,
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
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let TextDocumentItem { uri, text, version, .. } = params.text_document;
        if !self.tracks(&uri) {
            return;
        }

        self.apply_buffer_open(uri, text, version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if !self.tracks(&uri) {
            return;
        }

        let Some(change) = params.content_changes.into_iter().next_back() else {
            return;
        };

        self.apply_buffer_change(uri, change.text, params.text_document.version).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if !self.tracks(&params.text_document.uri) {
            return;
        }

        self.apply_buffer_close(params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if !self.tracks(&params.text_document.uri) {
            return;
        }

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
                Some(folding_range::compute(&analysis))
            });

            Ok(result.flatten())
        })
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> JsonRpcResult<Option<Vec<TextEdit>>> {
        traced("formatting", || {
            if !self.config.formatter {
                return Ok(None);
            }

            let php_version = self.config.configuration.php_version;
            let settings = self.config.configuration.formatter.settings;

            Ok(self
                .with_file(&params.text_document.uri, |file, _| formatting::compute(file, php_version, settings))
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
                hover::compute(ws.service.codebase(), analysis.resolved(), &file, offset)
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
                definition::compute(&ws.database, ws.service.codebase(), analysis.resolved(), offset)
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
                Some(selection_range::compute(&analysis, &file, &offsets))
            });

            Ok(result.flatten())
        })
    }

    async fn code_action(&self, params: CodeActionParams) -> JsonRpcResult<Option<CodeActionResponse>> {
        traced("code_action", || {
            let result = self.with_workspace(|ws| Some(code_action::compute(ws, &params)));
            Ok(result.flatten())
        })
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> JsonRpcResult<Option<SemanticTokensResult>> {
        traced("semantic_tokens_full", || {
            Ok(self.with_file(&params.text_document.uri, |file, _| {
                let data = semantic_tokens::compute(file);
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
                Some(references::compute(ws, &file, offset, include_decl))
            });

            Ok(result.flatten())
        })
    }

    async fn symbol(&self, params: WorkspaceSymbolParams) -> JsonRpcResult<Option<WorkspaceSymbolResponse>> {
        traced("workspace_symbol", || {
            Ok(self.with_workspace(|ws| workspace_symbol::compute(&ws.database, ws.service.codebase(), &params.query)))
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
                signature_help::compute(ws.service.codebase(), analysis.resolved(), &file, offset)
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
                Some(inlay_hint::compute(ws.service.codebase(), analysis.resolved(), &file, start, end))
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
                rename::prepare(analysis.resolved(), &file, offset)
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
                rename::compute(ws, &file, offset, new_name)
            });

            Ok(result.flatten())
        })
    }

    async fn document_link(&self, params: DocumentLinkParams) -> JsonRpcResult<Option<Vec<DocumentLink>>> {
        traced("document_link", || {
            let result = self.with_workspace(|ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                Some(document_link::compute(&ws.database, ws.service.codebase(), &file))
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
                Some(code_lens::compute(ws, &file, id))
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
                let analysis = ws.file_analysis_for(file_id)?;
                Some(completion::compute(
                    &ws.database,
                    ws.service.codebase(),
                    type_index.as_ref(),
                    analysis.resolved(),
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
