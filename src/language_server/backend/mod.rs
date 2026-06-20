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
use crate::language_server::capabilities::rename;
use crate::language_server::capabilities::server_capabilities;
use crate::language_server::codec;
use crate::language_server::position::offset_at_position;
use crate::language_server::state::BackendState;
use crate::language_server::workspace::workspace_roots;

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

        self.with_registry(|registry| registry.tracks(path.as_ref())).unwrap_or(true)
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> JsonRpcResult<InitializeResult> {
        let roots = match &self.config.workspace_override {
            Some(root) => vec![root.clone()],
            None => workspace_roots(&params),
        };
        if roots.is_empty() {
            self.client.log_message(MessageType::WARNING, "mago-server: no workspace root provided").await;
        } else {
            for root in &roots {
                self.client
                    .log_message(MessageType::INFO, format!("mago-server: workspace root = {}", root.display()))
                    .await;
            }

            self.bootstrap(roots).await;
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

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        for removed in params.event.removed {
            if let Some(path) = removed.uri.to_file_path() {
                self.remove_workspace_folder(path.into_owned()).await;
            }
        }
        for added in params.event.added {
            if let Some(path) = added.uri.to_file_path() {
                self.add_workspace_folder(path.into_owned()).await;
            }
        }
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
            let result = self.with_workspace_mut_for_uri(&params.text_document.uri, |ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let ranges = ws.server.get_folding_ranges(file.id);
                Some(codec::folding_ranges(&file, ranges))
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
                .with_file(&params.text_document.uri, |file, ws| {
                    ws.server.get_formatting(file.id).map(|document| codec::formatting(file, document))
                })
                .flatten()
                .map(|edit| vec![edit]))
        })
    }

    async fn hover(&self, params: HoverParams) -> JsonRpcResult<Option<Hover>> {
        traced("hover", || {
            let TextDocumentPositionParams { text_document, position } = params.text_document_position_params;
            let result = self.with_workspace_mut_for_uri(&text_document.uri, |ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                let info = ws.server.get_context(file.id, offset)?;
                Some(codec::hover(&file, info))
            });

            Ok(result.flatten())
        })
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> JsonRpcResult<Option<GotoDefinitionResponse>> {
        traced("goto_definition", || {
            let TextDocumentPositionParams { text_document, position } = params.text_document_position_params;
            let result = self.with_workspace_mut_for_uri(&text_document.uri, |ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                let location = ws.server.get_definition(file.id, offset)?;
                codec::location(ws.database(), &location).map(GotoDefinitionResponse::Scalar)
            });

            Ok(result.flatten())
        })
    }

    async fn selection_range(&self, params: SelectionRangeParams) -> JsonRpcResult<Option<Vec<SelectionRange>>> {
        traced("selection_range", || {
            let result = self.with_workspace_mut_for_uri(&params.text_document.uri, |ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let offsets: Vec<u32> = params.positions.iter().map(|p| offset_at_position(&file, *p)).collect();
                let ranges = ws.server.get_selection_ranges(file.id, &offsets);
                Some(codec::selection_ranges(&file, ranges))
            });

            Ok(result.flatten())
        })
    }

    async fn code_action(&self, params: CodeActionParams) -> JsonRpcResult<Option<CodeActionResponse>> {
        traced("code_action", || {
            let result = self.with_workspace_for_uri(&params.text_document.uri, |ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let start = offset_at_position(&file, params.range.start);
                let end = offset_at_position(&file, params.range.end);
                let actions = ws.server.get_code_actions(file.id, start, end);
                Some(codec::code_actions(ws.database(), actions))
            });
            Ok(result.flatten())
        })
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> JsonRpcResult<Option<SemanticTokensResult>> {
        traced("semantic_tokens_full", || {
            Ok(self
                .with_workspace_mut_for_uri(&params.text_document.uri, |ws| {
                    let file = file_for_uri(ws, &params.text_document.uri)?;
                    let data = codec::semantic_tokens(&file, ws.server.get_semantic_tokens(file.id));
                    Some(SemanticTokensResult::Tokens(SemanticTokens { result_id: None, data }))
                })
                .flatten())
        })
    }

    async fn references(&self, params: ReferenceParams) -> JsonRpcResult<Option<Vec<Location>>> {
        traced("references", || {
            let TextDocumentPositionParams { text_document, position } = params.text_document_position;
            let include_decl = params.context.include_declaration;
            let result = self.with_workspace_mut_for_uri(&text_document.uri, |ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                let references = ws.server.get_references(file.id, offset, include_decl);
                Some(codec::locations(ws.database(), &references))
            });

            Ok(result.flatten())
        })
    }

    async fn symbol(&self, params: WorkspaceSymbolParams) -> JsonRpcResult<Option<WorkspaceSymbolResponse>> {
        traced("workspace_symbol", || {
            Ok(self.with_registry(|registry| {
                let mut all = Vec::new();
                for ws in registry.iter() {
                    let items = ws.server.get_symbols(&params.query);
                    all.extend(codec::symbols(ws.database(), items));
                }
                WorkspaceSymbolResponse::Flat(all)
            }))
        })
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> JsonRpcResult<Option<SignatureHelp>> {
        traced("signature_help", || {
            if !self.config.analyzer {
                return Ok(None);
            }
            let TextDocumentPositionParams { text_document, position } = params.text_document_position_params;
            let result = self.with_workspace_mut_for_uri(&text_document.uri, |ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                ws.server.get_signature_help(file.id, offset).map(codec::signature_help)
            });

            Ok(result.flatten())
        })
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> JsonRpcResult<Option<Vec<InlayHint>>> {
        traced("inlay_hint", || {
            if !self.config.analyzer {
                return Ok(None);
            }

            let result = self.with_workspace_mut_for_uri(&params.text_document.uri, |ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let start = offset_at_position(&file, params.range.start);
                let end = offset_at_position(&file, params.range.end);
                let hints = ws.server.get_inlay_hints(file.id, start, end);
                Some(codec::inlay_hints(&file, hints))
            });

            Ok(result.flatten())
        })
    }

    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> JsonRpcResult<Option<PrepareRenameResponse>> {
        traced("prepare_rename", || {
            let result = self.with_workspace_mut_for_uri(&params.text_document.uri, |ws| {
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
            let result = self.with_workspace_mut_for_uri(&text_document.uri, |ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                rename::compute(ws, &file, offset, new_name)
            });

            Ok(result.flatten())
        })
    }

    async fn document_link(&self, params: DocumentLinkParams) -> JsonRpcResult<Option<Vec<DocumentLink>>> {
        traced("document_link", || {
            let result = self.with_workspace_for_uri(&params.text_document.uri, |ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let links = ws.server.get_document_links(file.id);
                Some(codec::document_links(&file, ws.database(), links))
            });

            Ok(result.flatten())
        })
    }

    async fn code_lens(&self, params: CodeLensParams) -> JsonRpcResult<Option<Vec<CodeLens>>> {
        traced("code_lens", || {
            if !self.config.analyzer {
                return Ok(None);
            }

            let result = self.with_workspace_mut_for_uri(&params.text_document.uri, |ws| {
                let file = file_for_uri(ws, &params.text_document.uri)?;
                let lenses = ws.server.get_lens(file.id);
                Some(codec::code_lens(&file, lenses))
            });

            Ok(result.flatten())
        })
    }

    async fn completion(&self, params: CompletionParams) -> JsonRpcResult<Option<CompletionResponse>> {
        traced("completion", || {
            let TextDocumentPositionParams { text_document, position } = params.text_document_position;
            let result = self.with_workspace_mut_for_uri(&text_document.uri, |ws| {
                let file = file_for_uri(ws, &text_document.uri)?;
                let offset = offset_at_position(&file, position);
                Some(codec::completion(&file, ws.server.get_completion(file.id, offset)))
            });

            Ok(result.flatten())
        })
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }
}
