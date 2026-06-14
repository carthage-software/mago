//! LSP capability handlers.
//!
//! Each module here implements a single LSP request as a pure function
//! against [`super::state::WorkspaceState`]. The handlers don't know about
//! `tower-lsp`, the JSON-RPC layer, or the [`crate::backend::Backend`]
//! mutex; the dispatcher in `backend.rs` is responsible for locking and
//! plumbing.
//!
//! Adding a new capability is mechanical:
//!
//! 1. Drop a `capabilities/<name>.rs` with one public function that takes
//!    `&WorkspaceState` plus the LSP request params and returns the LSP
//!    response.
//! 2. Wire it through [`crate::backend::Backend`] alongside the others.
//! 3. Advertise the capability from [`server_capabilities`].

use tower_lsp_server::ls_types::CodeActionKind;
use tower_lsp_server::ls_types::CodeActionOptions;
use tower_lsp_server::ls_types::CodeActionProviderCapability;
use tower_lsp_server::ls_types::CodeLensOptions;
use tower_lsp_server::ls_types::CompletionOptions;
use tower_lsp_server::ls_types::DocumentLinkOptions;
use tower_lsp_server::ls_types::FoldingRangeProviderCapability;
use tower_lsp_server::ls_types::HoverProviderCapability;
use tower_lsp_server::ls_types::OneOf;
use tower_lsp_server::ls_types::RenameOptions;
use tower_lsp_server::ls_types::SelectionRangeProviderCapability;
use tower_lsp_server::ls_types::SemanticTokenType;
use tower_lsp_server::ls_types::SemanticTokensFullOptions;
use tower_lsp_server::ls_types::SemanticTokensLegend;
use tower_lsp_server::ls_types::SemanticTokensOptions;
use tower_lsp_server::ls_types::SemanticTokensServerCapabilities;
use tower_lsp_server::ls_types::ServerCapabilities;
use tower_lsp_server::ls_types::SignatureHelpOptions;
use tower_lsp_server::ls_types::TextDocumentSyncCapability;
use tower_lsp_server::ls_types::TextDocumentSyncKind;
use tower_lsp_server::ls_types::TextDocumentSyncOptions;
use tower_lsp_server::ls_types::TextDocumentSyncSaveOptions;
use tower_lsp_server::ls_types::WorkDoneProgressOptions;
use tower_lsp_server::ls_types::WorkspaceFoldersServerCapabilities;
use tower_lsp_server::ls_types::WorkspaceServerCapabilities;

pub mod rename;

/// Build the [`ServerCapabilities`] block to advertise during `initialize`.
///
/// Several capabilities are gated by [`super::ServerConfig`]: `--no-analyzer`
/// drops type-aware features (signature help, inlay hints, code lenses,
/// completion's member paths still work syntactically but yield empty
/// results), `--no-formatter` drops `textDocument/formatting`, etc.
#[must_use]
pub fn server_capabilities(config: &super::ServerConfig) -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::FULL),
            will_save: Some(false),
            will_save_wait_until: Some(false),
            save: Some(TextDocumentSyncSaveOptions::Supported(true)),
        })),
        document_formatting_provider: config.formatter.then_some(OneOf::Left(true)),
        workspace_symbol_provider: Some(OneOf::Left(true)),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        signature_help_provider: config.analyzer.then(|| SignatureHelpOptions {
            trigger_characters: Some(vec!["(".into(), ",".into()]),
            retrigger_characters: None,
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        inlay_hint_provider: config.analyzer.then_some(OneOf::Left(true)),
        document_link_provider: Some(DocumentLinkOptions {
            resolve_provider: Some(false),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        code_lens_provider: config.analyzer.then_some(CodeLensOptions { resolve_provider: Some(false) }),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec!["$".into(), ":".into(), "\\".into(), ">".into()]),
            all_commit_characters: None,
            work_done_progress_options: WorkDoneProgressOptions::default(),
            completion_item: None,
        }),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
            work_done_progress_options: WorkDoneProgressOptions::default(),
            resolve_provider: Some(false),
        })),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions::default(),
                legend: SemanticTokensLegend {
                    token_types: vec![
                        SemanticTokenType::KEYWORD,
                        SemanticTokenType::COMMENT,
                        SemanticTokenType::STRING,
                        SemanticTokenType::NUMBER,
                        SemanticTokenType::OPERATOR,
                        SemanticTokenType::VARIABLE,
                        SemanticTokenType::FUNCTION,
                        SemanticTokenType::TYPE,
                        SemanticTokenType::NAMESPACE,
                        SemanticTokenType::PARAMETER,
                        SemanticTokenType::PROPERTY,
                    ],
                    token_modifiers: vec![],
                },
                range: Some(false),
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: None,
        }),
        ..ServerCapabilities::default()
    }
}
