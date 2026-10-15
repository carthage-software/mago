//! Completion and call signatures for the current document.

use tower_lsp_server::gen_lsp_types::{CompletionContext, CompletionResponse, Position, SignatureHelp};

use crate::language_server::codec;

use super::query::DocumentQuery;

impl DocumentQuery<'_> {
    pub fn completion(self, position: Position, context: Option<CompletionContext>) -> CompletionResponse {
        let Some(offset) = self.checked_offset(position) else {
            return CompletionResponse::CompletionItemList(Vec::new());
        };

        let after_space = context.is_some_and(|context| context.trigger_character.as_deref() == Some(" "));
        let items = self.workspace.core.server.get_completion(self.file.id, offset, after_space);

        codec::completion(
            &self.file,
            items,
            self.preferences.completion_snippets,
            self.preferences.markdown_completion.then_some(self.preferences.hover_code_language),
            self.preferences.completion_as_is,
            self.preferences.completion_insert_mode,
            self.preferences.completion_label_details,
            self.preferences.completion_deprecated_tag,
        )
    }

    pub fn signature(self, position: Position) -> Option<SignatureHelp> {
        let offset = self.checked_offset(position)?;
        let signatures = self.workspace.core.server.get_signatures(self.file.id, offset);
        codec::signature_help(
            signatures,
            self.preferences.signature_offsets,
            self.preferences.signature_active_parameter,
            self.preferences.markdown_signature,
        )
    }
}
