//! Encode workspace edits with the client's accepted document versions.

use tower_lsp_server::gen_lsp_types::{
    DocumentChange, Edit, OptionalVersionedTextDocumentIdentifier, TextDocumentEdit, TextDocumentIdentifier,
    WorkspaceEdit,
};

use super::Backend;

impl Backend {
    pub(super) fn workspace_edit(&self, mut edit: WorkspaceEdit) -> WorkspaceEdit {
        if !self.client_preferences().versioned_edits || edit.document_changes.is_some() {
            return edit;
        }

        let _ = self.with_registry(|registry| {
            let Some(changes) = edit.changes.take() else { return };
            let mut documents: Vec<TextDocumentEdit> = changes
                .into_iter()
                .map(|(uri, edits)| TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        version: registry.document_version(&uri),
                        text_document_identifier: TextDocumentIdentifier { uri },
                    },
                    edits: edits.into_iter().map(Edit::TextEdit).collect(),
                })
                .collect();

            documents.sort_unstable_by(|left, right| {
                left.text_document
                    .text_document_identifier
                    .uri
                    .as_str()
                    .cmp(right.text_document.text_document_identifier.uri.as_str())
            });

            edit.document_changes = Some(documents.into_iter().map(DocumentChange::TextDocumentEdit).collect());
        });

        edit
    }
}
