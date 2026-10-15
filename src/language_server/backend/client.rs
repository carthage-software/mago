//! Response formats and optional protocol support chosen by the client.

use tower_lsp_server::gen_lsp_types::{
    CompletionItemTag, ExecuteCommandOptions, InitializeParams, InsertTextMode, MarkupKind, ServerCapabilities,
};

use crate::language_server::ServerConfig;
use crate::language_server::diagnostics::DiagnosticPreferences;
use crate::language_server::{capabilities, codec};

#[derive(Clone, Copy, Default)]
pub(super) struct ClientPreferences {
    pub markdown_hover: bool,
    pub hover_code_language: codec::HoverCodeLanguage,
    pub hierarchical_symbols: bool,
    pub dynamic_watchers: bool,
    pub code_lens_refresh: bool,
    pub completion_snippets: bool,
    pub completion_label_details: bool,
    pub completion_deprecated_tag: bool,
    pub completion_as_is: bool,
    pub completion_insert_mode: Option<InsertTextMode>,
    pub markdown_completion: bool,
    pub signature_offsets: bool,
    pub markdown_signature: bool,
    pub signature_active_parameter: bool,
    pub code_action_literals: bool,
    pub preferred_code_actions: bool,
    pub apply_edits: bool,
    pub versioned_edits: bool,
    pub prepare_rename: bool,
    pub diagnostics: DiagnosticPreferences,
}

impl ClientPreferences {
    pub(super) fn server_capabilities(self, config: &ServerConfig) -> ServerCapabilities {
        let mut capabilities = capabilities::server_capabilities(config);
        if !self.prepare_rename && capabilities.rename_provider.is_some() {
            capabilities.rename_provider = Some(true.into());
        }
        if !self.code_action_literals && capabilities.code_action_provider.is_some() {
            capabilities.code_action_provider = self.apply_edits.then_some(true.into());
            capabilities.execute_command_provider = self.apply_edits.then(|| ExecuteCommandOptions {
                commands: vec![codec::APPLY_CODE_ACTION.into()],
                work_done_progress_options: Default::default(),
            });
        }
        capabilities
    }
}

impl From<&InitializeParams> for ClientPreferences {
    fn from(params: &InitializeParams) -> Self {
        let capabilities = &params.capabilities;
        let document = capabilities.text_document.as_ref();
        let code_action = document.and_then(|document| document.code_action.as_ref());
        let completion_capabilities = document.and_then(|document| document.completion.as_ref());
        let completion = completion_capabilities.and_then(|completion| completion.completion_item.as_ref());
        let completion_insert_mode =
            completion.and_then(|item| item.insert_text_mode_support.as_ref()).and_then(|support| {
                [InsertTextMode::AdjustIndentation, InsertTextMode::AsIs]
                    .into_iter()
                    .find(|mode| support.value_set.contains(mode))
            });
        let signature = document
            .and_then(|document| document.signature_help.as_ref())
            .and_then(|help| help.signature_information.as_ref());
        Self {
            diagnostics: DiagnosticPreferences::from_capabilities(
                document.and_then(|document| document.publish_diagnostics.as_ref()),
                document.and_then(|document| document.diagnostic.as_ref()),
            ),
            hover_code_language: codec::HoverCodeLanguage::from_options(params.initialization_options.as_ref()),
            code_action_literals: code_action.and_then(|action| action.code_action_literal_support.as_ref()).is_some(),
            preferred_code_actions: code_action.and_then(|action| action.is_preferred_support).unwrap_or(false),
            apply_edits: capabilities.workspace.as_ref().and_then(|workspace| workspace.apply_edit).unwrap_or(false),
            versioned_edits: capabilities
                .workspace
                .as_ref()
                .and_then(|workspace| workspace.workspace_edit.as_ref())
                .and_then(|edit| edit.document_changes)
                .unwrap_or(false),
            prepare_rename: document
                .and_then(|document| document.rename.as_ref())
                .and_then(|rename| rename.prepare_support)
                .unwrap_or(false),
            completion_snippets: completion.and_then(|item| item.snippet_support).unwrap_or(false),
            completion_label_details: completion.and_then(|item| item.label_details_support).unwrap_or(false),
            completion_deprecated_tag: completion
                .and_then(|item| item.tag_support.as_ref())
                .is_some_and(|support| support.value_set.contains(&CompletionItemTag::Deprecated)),
            completion_as_is: completion_insert_mode
                .or_else(|| completion_capabilities.and_then(|completion| completion.insert_text_mode))
                == Some(InsertTextMode::AsIs),
            completion_insert_mode,
            markdown_completion: completion.and_then(|item| item.documentation_format.as_ref()).and_then(|formats| {
                formats.iter().find(|format| matches!(format, MarkupKind::Markdown | MarkupKind::PlainText))
            }) == Some(&MarkupKind::Markdown),
            markdown_signature: signature.and_then(|signature| signature.documentation_format.as_ref()).and_then(
                |formats| formats.iter().find(|format| matches!(format, MarkupKind::Markdown | MarkupKind::PlainText)),
            ) == Some(&MarkupKind::Markdown),
            signature_offsets: signature
                .and_then(|signature| signature.parameter_information.as_ref())
                .and_then(|parameter| parameter.label_offset_support)
                .unwrap_or(false),
            signature_active_parameter: signature
                .and_then(|signature| signature.active_parameter_support)
                .unwrap_or(false),
            markdown_hover: document
                .and_then(|document| document.hover.as_ref())
                .and_then(|hover| hover.content_format.as_ref())
                .and_then(|formats| {
                    formats.iter().find(|format| matches!(format, MarkupKind::Markdown | MarkupKind::PlainText))
                })
                == Some(&MarkupKind::Markdown),
            hierarchical_symbols: document
                .and_then(|document| document.document_symbol.as_ref())
                .and_then(|symbols| symbols.hierarchical_document_symbol_support)
                .unwrap_or(false),
            dynamic_watchers: capabilities
                .workspace
                .as_ref()
                .and_then(|workspace| workspace.did_change_watched_files.as_ref())
                .and_then(|watchers| watchers.dynamic_registration)
                .unwrap_or(false),
            code_lens_refresh: capabilities
                .workspace
                .as_ref()
                .and_then(|workspace| workspace.code_lens.as_ref())
                .and_then(|lenses| lenses.refresh_support)
                .unwrap_or(false),
        }
    }
}
