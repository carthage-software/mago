//! Format source and build edits for code actions and rename.

use tower_lsp_server::gen_lsp_types::{
    CodeActionContext, CodeActionResponse, Position, PrepareRenameResult, Range, TextEdit, Uri, WorkspaceEdit,
};

use crate::language_server::capabilities::rename;
use crate::language_server::codec;

use super::query::DocumentQuery;

impl DocumentQuery<'_> {
    pub fn formatting(self) -> Option<Vec<TextEdit>> {
        let formatted = self.workspace.core.server.get_formatting(self.file.id)?;
        Some(vec![codec::formatting(&self.file, formatted)])
    }

    pub fn range_formatting(self, range: Range) -> Option<Vec<TextEdit>> {
        let range = self.checked_range(range)?;
        let edit = self.workspace.core.server.get_range_formatting(self.file.id, range)?;
        Some(vec![codec::text_edit(&self.file, edit)])
    }

    pub fn code_actions(self, uri: &Uri, range: Range, context: CodeActionContext) -> Vec<CodeActionResponse> {
        let Some(range) = self.checked_range(range) else { return Vec::new() };
        let only = context.only.as_ref().map(|kinds| kinds.iter().map(|kind| kind.as_str()).collect::<Vec<_>>());
        let actions =
            self.workspace.core.server.get_code_actions(self.file.id, range.start, range.end, only.as_deref());
        codec::code_actions(
            self.workspace.database(),
            |file| self.workspace.document_uri(file),
            actions,
            codec::CodeActionSource {
                uri,
                file: &self.file,
                diagnostics: &context.diagnostics,
                markdown_diagnostics: self.preferences.diagnostics.supports_markdown(),
            },
            self.preferences.code_action_literals,
            self.preferences.preferred_code_actions,
        )
    }

    pub fn prepare_rename(self, position: Position) -> Option<Result<PrepareRenameResult, String>> {
        let offset = self.checked_symbol_offset(position)?;
        rename::prepare(self.workspace, &self.file, offset)
    }

    pub fn rename(self, position: Position, name: String) -> Option<Result<WorkspaceEdit, String>> {
        let offset = self.checked_symbol_offset(position)?;
        rename::compute(self.workspace, &self.file, offset, name)
    }
}
