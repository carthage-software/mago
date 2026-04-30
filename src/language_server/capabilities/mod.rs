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

use tower_lsp::lsp_types::CodeActionKind;
use tower_lsp::lsp_types::CodeActionOptions;
use tower_lsp::lsp_types::CodeActionProviderCapability;
use tower_lsp::lsp_types::CodeLensOptions;
use tower_lsp::lsp_types::CompletionOptions;
use tower_lsp::lsp_types::DocumentLinkOptions;
use tower_lsp::lsp_types::FoldingRangeProviderCapability;
use tower_lsp::lsp_types::HoverProviderCapability;
use tower_lsp::lsp_types::OneOf;
use tower_lsp::lsp_types::RenameOptions;
use tower_lsp::lsp_types::SelectionRangeProviderCapability;
use tower_lsp::lsp_types::SemanticTokenType;
use tower_lsp::lsp_types::SemanticTokensFullOptions;
use tower_lsp::lsp_types::SemanticTokensLegend;
use tower_lsp::lsp_types::SemanticTokensOptions;
use tower_lsp::lsp_types::SemanticTokensServerCapabilities;
use tower_lsp::lsp_types::ServerCapabilities;
use tower_lsp::lsp_types::SignatureHelpOptions;
use tower_lsp::lsp_types::TextDocumentSyncCapability;
use tower_lsp::lsp_types::TextDocumentSyncKind;
use tower_lsp::lsp_types::TextDocumentSyncOptions;
use tower_lsp::lsp_types::TextDocumentSyncSaveOptions;
use tower_lsp::lsp_types::WorkDoneProgressOptions;

pub mod code_action;
pub mod code_lens;
pub mod completion;
pub mod definition;
pub mod document_link;
pub mod folding_range;
pub mod formatting;
pub mod hover;
pub mod inlay_hint;
pub mod lookup;
pub mod references;
pub mod rename;
pub mod selection_range;
pub mod semantic_tokens;
pub mod signature_help;
pub mod workspace_symbol;

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
                    token_types: semantic_tokens::TOKEN_TYPES.to_vec(),
                    token_modifiers: vec![],
                },
                range: Some(false),
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),
        ..ServerCapabilities::default()
    }
}

/// Re-export the semantic-token type ordering, used both in the legend and by
/// the encoder.
#[allow(dead_code)]
pub fn token_type_index(ty: SemanticTokenType) -> Option<u32> {
    semantic_tokens::TOKEN_TYPES.iter().position(|t| *t == ty).map(|i| i as u32)
}
