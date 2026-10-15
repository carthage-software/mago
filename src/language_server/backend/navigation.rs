//! Navigation and hover queries against the current document.

use tower_lsp_server::gen_lsp_types::{
    Definition, DefinitionResponse, DocumentHighlight, Hover, ImplementationResponse, Location, Position,
    TypeDefinitionResponse,
};

use crate::language_server::codec;

use super::query::DocumentQuery;

impl DocumentQuery<'_> {
    pub fn hover(self, position: Position) -> Option<Hover> {
        let offset = self.checked_symbol_offset(position)?;
        let info = self.workspace.core.server.get_context(self.file.id, offset)?;
        Some(codec::hover(&self.file, info, self.preferences.markdown_hover, self.preferences.hover_code_language))
    }

    pub fn definitions(self, position: Position) -> Option<DefinitionResponse> {
        let offset = self.checked_symbol_offset(position)?;
        let locations = self.workspace.core.server.get_definitions(self.file.id, offset);
        let mut locations = self.workspace.locations(&locations);
        match locations.len() {
            0 => None,
            1 => locations.pop().map(|location| Definition::Location(location).into()),
            _ => Some(Definition::LocationList(locations).into()),
        }
    }

    pub fn references(self, position: Position, include_declaration: bool) -> Option<Vec<Location>> {
        let offset = self.checked_symbol_offset(position)?;
        let references = self.workspace.core.server.get_references(self.file.id, offset, include_declaration);
        Some(self.workspace.locations(&references))
    }

    pub fn type_definitions(self, position: Position) -> Option<TypeDefinitionResponse> {
        let offset = self.checked_symbol_offset(position)?;
        let locations = self.workspace.core.server.get_type_definitions(self.file.id, offset);
        let locations = self.workspace.locations(&locations);
        (!locations.is_empty()).then_some(Definition::LocationList(locations).into())
    }

    pub fn implementations(self, position: Position) -> Option<ImplementationResponse> {
        let offset = self.checked_symbol_offset(position)?;
        let locations = self.workspace.core.server.get_implementations(self.file.id, offset);
        let locations = self.workspace.locations(&locations);
        (!locations.is_empty()).then_some(Definition::LocationList(locations).into())
    }

    pub fn highlights(self, position: Position) -> Option<Vec<DocumentHighlight>> {
        let offset = self.checked_symbol_offset(position)?;
        let highlights = self.workspace.core.server.get_document_highlights(self.file.id, offset);
        (!highlights.is_empty()).then(|| codec::document_highlights(&self.file, highlights))
    }
}
