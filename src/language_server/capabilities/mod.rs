//! LSP capability advertisement and workspace-level handlers.
//!
//! The backend owns session state and document routing. These handlers translate
//! workspace queries into protocol values without taking the backend lock.

use tower_lsp_server::gen_lsp_types::CodeActionKind;
use tower_lsp_server::gen_lsp_types::CodeActionOptions;
use tower_lsp_server::gen_lsp_types::CodeLensOptions;
use tower_lsp_server::gen_lsp_types::CompletionOptions;
use tower_lsp_server::gen_lsp_types::DocumentLinkOptions;
use tower_lsp_server::gen_lsp_types::RenameOptions;
use tower_lsp_server::gen_lsp_types::ServerCapabilities;
use tower_lsp_server::gen_lsp_types::SignatureHelpOptions;
use tower_lsp_server::gen_lsp_types::TextDocumentSyncKind;
use tower_lsp_server::gen_lsp_types::TextDocumentSyncOptions;
use tower_lsp_server::gen_lsp_types::WorkDoneProgressOptions;
use tower_lsp_server::gen_lsp_types::WorkspaceFoldersServerCapabilities;
use tower_lsp_server::gen_lsp_types::WorkspaceOptions;

use crate::config::language_server::Provider;

pub mod call_hierarchy;
pub mod rename;
pub mod type_hierarchy;

/// Build capabilities allowed by server settings. The session adapts their
/// response formats to client support before returning them in `initialize`.
///
/// Several capabilities are gated by [`super::ServerConfig`]: `--no-analyzer`
/// drops type-aware features (signature help, inlay hints, code lenses,
/// completion's member paths still work syntactically but yield empty
/// results), `--no-formatter` drops `textDocument/formatting`, etc.
#[must_use]
pub fn server_capabilities(config: &super::ServerConfig) -> ServerCapabilities {
    let mut capabilities = ServerCapabilities {
        text_document_sync: Some(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::Full),
                will_save: Some(false),
                will_save_wait_until: Some(false),
                save: Some(true.into()),
            }
            .into(),
        ),
        document_formatting_provider: config.supports_provider(Provider::Formatting).then_some(true.into()),
        document_range_formatting_provider: config.supports_provider(Provider::RangeFormatting).then_some(true.into()),
        workspace_symbol_provider: Some(true.into()),
        document_symbol_provider: Some(true.into()),
        folding_range_provider: Some(true.into()),
        hover_provider: Some(true.into()),
        definition_provider: Some(true.into()),
        type_definition_provider: config.supports_provider(Provider::TypeDefinition).then_some(true.into()),
        document_highlight_provider: Some(true.into()),
        implementation_provider: config.supports_provider(Provider::Implementation).then_some(true.into()),
        call_hierarchy_provider: config.supports_provider(Provider::CallHierarchy).then_some(true.into()),
        type_hierarchy_provider: (config.supports_provider(Provider::TypeHierarchy)
            && config.configuration.language_server.is_enabled(Provider::TypeHierarchy))
        .then_some(true.into()),
        references_provider: Some(true.into()),
        selection_range_provider: Some(true.into()),
        rename_provider: Some(
            RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }
            .into(),
        ),
        signature_help_provider: config.supports_provider(Provider::SignatureHelp).then(|| SignatureHelpOptions {
            trigger_characters: Some(vec!["(".into(), ",".into()]),
            retrigger_characters: None,
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        inlay_hint_provider: config.supports_provider(Provider::InlayHint).then_some(true.into()),
        document_link_provider: Some(DocumentLinkOptions {
            resolve_provider: Some(false),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        code_lens_provider: config.supports_provider(Provider::CodeLens).then_some(CodeLensOptions {
            resolve_provider: Some(true),
            work_done_progress_options: Default::default(),
        }),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec!["$".into(), ":".into(), "\\".into(), ">".into(), " ".into()]),
            all_commit_characters: None,
            work_done_progress_options: WorkDoneProgressOptions::default(),
            completion_item: None,
        }),
        code_action_provider: Some(
            CodeActionOptions {
                documentation: None,
                code_action_kinds: Some(vec![
                    CodeActionKind::QuickFix,
                    CodeActionKind::RefactorRewrite,
                    CodeActionKind::RefactorExtract,
                ]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: Some(false),
            }
            .into(),
        ),
        workspace: Some(WorkspaceOptions {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(true.into()),
            }),
            file_operations: None,
            text_document_content: None,
        }),
        ..ServerCapabilities::default()
    };

    for provider in &config.configuration.language_server.disable {
        match provider {
            Provider::Formatting => capabilities.document_formatting_provider = None,
            Provider::RangeFormatting => capabilities.document_range_formatting_provider = None,
            Provider::Hover => capabilities.hover_provider = None,
            Provider::Definition => capabilities.definition_provider = None,
            Provider::TypeDefinition => capabilities.type_definition_provider = None,
            Provider::DocumentHighlight => capabilities.document_highlight_provider = None,
            Provider::Implementation => capabilities.implementation_provider = None,
            Provider::TypeHierarchy => capabilities.type_hierarchy_provider = None,
            Provider::CallHierarchy => capabilities.call_hierarchy_provider = None,
            Provider::References => capabilities.references_provider = None,
            Provider::Rename => capabilities.rename_provider = None,
            Provider::Completion => capabilities.completion_provider = None,
            Provider::FoldingRange => capabilities.folding_range_provider = None,
            Provider::SelectionRange => capabilities.selection_range_provider = None,
            Provider::CodeAction => capabilities.code_action_provider = None,
            Provider::WorkspaceSymbol => capabilities.workspace_symbol_provider = None,
            Provider::DocumentSymbol => capabilities.document_symbol_provider = None,
            Provider::SignatureHelp => capabilities.signature_help_provider = None,
            Provider::InlayHint => capabilities.inlay_hint_provider = None,
            Provider::DocumentLink => capabilities.document_link_provider = None,
            Provider::CodeLens => capabilities.code_lens_provider = None,
        }
    }

    capabilities
}
