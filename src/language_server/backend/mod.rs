//! LSP request dispatch. Session setup, workspace loading, document sync,
//! and request routing have separate modules.

use std::sync::Arc;
use std::sync::OnceLock;

use foldhash::HashSet;
use mago_server::capabilities::workspace_symbol;
use tower_lsp_server::Client;
use tower_lsp_server::LanguageServer;
use tower_lsp_server::gen_lsp_types::ImplementationParams;
use tower_lsp_server::gen_lsp_types::ImplementationResponse;
use tower_lsp_server::gen_lsp_types::TypeDefinitionParams;
use tower_lsp_server::gen_lsp_types::TypeDefinitionResponse;
use tower_lsp_server::gen_lsp_types::*;
use tower_lsp_server::jsonrpc::Result as JsonRpcResult;

use crate::config::language_server::LanguageServerConfiguration;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;
use crate::language_server::capabilities::type_hierarchy;
use crate::language_server::codec;
use crate::language_server::uri::UriFilePath;

mod buffers;
mod client;
mod code_lens;
mod code_lens_refresh;
mod commands;
mod completion;
mod editing;
mod hierarchy;
mod lifecycle;
mod navigation;
mod publication;
mod query;
mod routing;
mod session;
mod structure;
mod sync;
mod watchers;
mod workspace_edit;

use routing::traced;
use session::Session;

pub struct Backend {
    client: Client,
    config: Arc<ServerConfig>,
    session: OnceLock<Session>,
}

impl Backend {
    #[must_use]
    pub fn new(client: Client, config: Arc<ServerConfig>) -> Self {
        Self { client, config, session: OnceLock::new() }
    }

    /// Advertise a provider if at least one loaded workspace enables it.
    fn providers(&self) -> LanguageServerConfiguration {
        self.with_registry(|registry| session::providers(registry, &self.config))
            .unwrap_or_else(|| self.config.configuration.language_server.clone())
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> JsonRpcResult<InitializeResult> {
        self.start_session(params).await
    }

    async fn initialized(&self, _: InitializedParams) {
        self.publish_initial_diagnostics().await;
        self.refresh_watchers();
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        // Workspace files and launch options own the server settings.
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

        self.refresh_watchers();
        self.notify_capability_change().await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let TextDocumentItem { uri, text, version, .. } = params.text_document;
        self.apply_buffer_open(uri, text, version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.text_document_identifier.uri;
        let Some(change) = params.content_changes.into_iter().next_back() else {
            return;
        };

        let TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(change) = change else {
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
        self.apply_disk_changes(params.changes).await;
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> JsonRpcResult<Option<Vec<FoldingRange>>> {
        self.query(Provider::FoldingRange, &params.text_document.uri, |document| Some(document.folds()))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> JsonRpcResult<Option<Vec<TextEdit>>> {
        self.query(Provider::Formatting, &params.text_document.uri, |document| document.formatting())
    }

    async fn range_formatting(&self, params: DocumentRangeFormattingParams) -> JsonRpcResult<Option<Vec<TextEdit>>> {
        self.query(Provider::RangeFormatting, &params.text_document.uri, |document| {
            document.range_formatting(params.range)
        })
    }

    async fn hover(&self, params: HoverParams) -> JsonRpcResult<Option<Hover>> {
        let params = params.text_document_position_params;
        self.query(Provider::Hover, &params.text_document.uri, |document| document.hover(params.position))
    }

    async fn goto_definition(&self, params: DefinitionParams) -> JsonRpcResult<Option<DefinitionResponse>> {
        let params = params.text_document_position_params;
        self.query(Provider::Definition, &params.text_document.uri, |document| document.definitions(params.position))
    }

    async fn selection_range(&self, params: SelectionRangeParams) -> JsonRpcResult<Option<Vec<SelectionRange>>> {
        self.query(Provider::SelectionRange, &params.text_document.uri, |document| {
            Some(document.selections(&params.positions))
        })
    }

    async fn code_action(&self, params: CodeActionParams) -> JsonRpcResult<Option<Vec<CodeActionResponse>>> {
        let preferences = self.client_preferences();
        if !preferences.code_action_literals && !preferences.apply_edits {
            return Ok(None);
        }

        let mut actions = self.query(Provider::CodeAction, &params.text_document.uri, |document| {
            Some(document.code_actions(&params.text_document.uri, params.range, params.context))
        })?;

        if let Some(actions) = &mut actions {
            for action in actions {
                if let CodeActionResponse::CodeAction(action) = action
                    && let Some(edit) = action.edit.take()
                {
                    action.edit = Some(self.workspace_edit(edit));
                }
            }
        }

        Ok(actions)
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> JsonRpcResult<Option<serde_json::Value>> {
        self.execute_code_action(params).await
    }

    async fn references(&self, params: ReferenceParams) -> JsonRpcResult<Option<Vec<Location>>> {
        let position = params.text_document_position_params;
        self.query(Provider::References, &position.text_document.uri, |document| {
            document.references(position.position, params.context.include_declaration)
        })
    }

    async fn goto_type_definition(
        &self,
        params: TypeDefinitionParams,
    ) -> JsonRpcResult<Option<TypeDefinitionResponse>> {
        let params = params.text_document_position_params;
        self.query(Provider::TypeDefinition, &params.text_document.uri, |document| {
            document.type_definitions(params.position)
        })
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> JsonRpcResult<Option<Vec<DocumentHighlight>>> {
        let params = params.text_document_position_params;
        self.query(Provider::DocumentHighlight, &params.text_document.uri, |document| {
            document.highlights(params.position)
        })
    }

    async fn goto_implementation(&self, params: ImplementationParams) -> JsonRpcResult<Option<ImplementationResponse>> {
        let params = params.text_document_position_params;
        self.query(Provider::Implementation, &params.text_document.uri, |document| {
            document.implementations(params.position)
        })
    }

    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> JsonRpcResult<Option<Vec<CallHierarchyItem>>> {
        let params = params.text_document_position_params;
        self.query(Provider::CallHierarchy, &params.text_document.uri, |document| {
            document.call_hierarchy(params.position)
        })
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> JsonRpcResult<Option<Vec<CallHierarchyIncomingCall>>> {
        self.incoming_calls_for(params.item)
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> JsonRpcResult<Option<Vec<CallHierarchyOutgoingCall>>> {
        self.outgoing_calls_for(params.item)
    }

    async fn prepare_type_hierarchy(
        &self,
        params: TypeHierarchyPrepareParams,
    ) -> JsonRpcResult<Option<Vec<TypeHierarchyItem>>> {
        let params = params.text_document_position_params;
        self.query(Provider::TypeHierarchy, &params.text_document.uri, |document| {
            document.type_hierarchy(params.position)
        })
    }

    async fn supertypes(&self, params: TypeHierarchySupertypesParams) -> JsonRpcResult<Option<Vec<TypeHierarchyItem>>> {
        self.related_types(params.item, type_hierarchy::Direction::Supertypes)
    }

    async fn subtypes(&self, params: TypeHierarchySubtypesParams) -> JsonRpcResult<Option<Vec<TypeHierarchyItem>>> {
        self.related_types(params.item, type_hierarchy::Direction::Subtypes)
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> JsonRpcResult<Option<DocumentSymbolResponse>> {
        self.query(Provider::DocumentSymbol, &params.text_document.uri, |document| {
            Some(document.symbols(&params.text_document.uri))
        })
    }

    async fn symbol(&self, params: WorkspaceSymbolParams) -> JsonRpcResult<Option<WorkspaceSymbolResponse>> {
        traced("workspace_symbol", || {
            if !self.providers().is_enabled(Provider::WorkspaceSymbol) {
                return Ok(None);
            }

            self.with_registry(|registry| {
                let mut all = Vec::new();
                let mut seen = HashSet::default();
                for ws in registry.iter() {
                    if !ws.configuration.language_server.is_enabled(Provider::WorkspaceSymbol) {
                        continue;
                    }

                    query::require_analysis(ws)?;
                    let items = ws.core.server.get_symbols(&params.query);
                    all.extend(codec::symbols(ws, items).into_iter().filter(|(_, symbol)| {
                        seen.insert((symbol.location.clone(), symbol.base_symbol_information.name.clone()))
                    }));
                }

                all.sort_by(|(left_rank, left), (right_rank, right)| {
                    left_rank.cmp(right_rank).then_with(|| {
                        symbol_name_order(&left.base_symbol_information.name, &right.base_symbol_information.name)
                    })
                });
                all.truncate(workspace_symbol::MAX_RESULTS);
                Ok(WorkspaceSymbolResponse::SymbolInformationList(all.into_iter().map(|(_, symbol)| symbol).collect()))
            })
            .transpose()
        })
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> JsonRpcResult<Option<SignatureHelp>> {
        let params = params.text_document_position_params;
        self.query(Provider::SignatureHelp, &params.text_document.uri, |document| document.signature(params.position))
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> JsonRpcResult<Option<Vec<InlayHint>>> {
        self.query(Provider::InlayHint, &params.text_document.uri, |document| Some(document.hints(params.range)))
    }

    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> JsonRpcResult<Option<PrepareRenameResult>> {
        self.query(Provider::Rename, &params.text_document.uri, |document| document.prepare_rename(params.position))?
            .transpose()
            .map_err(query::request_failed)
    }

    async fn rename(&self, params: RenameParams) -> JsonRpcResult<Option<WorkspaceEdit>> {
        let position = params.text_document_position_params;
        let edit = self.query(Provider::Rename, &position.text_document.uri, |document| {
            document.rename(position.position, params.new_name)
        })?;
        edit.transpose().map(|edit| edit.map(|edit| self.workspace_edit(edit))).map_err(query::request_failed)
    }

    async fn document_link(&self, params: DocumentLinkParams) -> JsonRpcResult<Option<Vec<DocumentLink>>> {
        self.query(Provider::DocumentLink, &params.text_document.uri, |document| Some(document.links()))
    }

    async fn code_lens(&self, params: CodeLensParams) -> JsonRpcResult<Option<Vec<CodeLens>>> {
        self.query(Provider::CodeLens, &params.text_document.uri, |document| {
            Some(document.lenses(&params.text_document.uri))
        })
    }

    async fn code_lens_resolve(&self, params: CodeLens) -> JsonRpcResult<CodeLens> {
        self.resolve_lens(params)
    }

    async fn completion(&self, params: CompletionParams) -> JsonRpcResult<Option<CompletionResponse>> {
        let context = params.context;
        let params = params.text_document_position_params;
        self.query(Provider::Completion, &params.text_document.uri, |document| {
            Some(document.completion(params.position, context))
        })
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        self.stop_session().await
    }
}

/// Short names ignoring case, then full names, as each workspace orders its own matches.
fn symbol_name_order(left: &str, right: &str) -> std::cmp::Ordering {
    short_symbol_name(left)
        .bytes()
        .map(|byte| byte.to_ascii_lowercase())
        .cmp(short_symbol_name(right).bytes().map(|byte| byte.to_ascii_lowercase()))
        .then_with(|| left.cmp(right))
}

fn short_symbol_name(name: &str) -> &str {
    match name.rsplit_once("::") {
        Some((_, member)) => member.trim_start_matches('$'),
        None => name.rsplit_once('\\').map_or(name, |(_, short)| short),
    }
}
