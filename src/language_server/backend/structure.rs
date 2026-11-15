//! Document outlines, ranges, hints, and links.

use tower_lsp_server::gen_lsp_types::{
    DocumentLink, DocumentSymbolResponse, FoldingRange, InlayHint, Position, Range, SelectionRange, Uri,
};

use mago_database::DatabaseReader;

use crate::language_server::codec;
use crate::language_server::position::position_at_offset;

use super::query::DocumentQuery;

impl DocumentQuery<'_> {
    pub fn folds(self) -> Vec<FoldingRange> {
        let ranges = self.workspace.core.server.get_folding_ranges(self.file.id);
        codec::folding_ranges(&self.file, ranges)
    }

    pub fn selections(self, positions: &[Position]) -> Vec<SelectionRange> {
        let offsets: Vec<_> = positions.iter().map(|position| self.offset(*position)).collect();
        let ranges = self.workspace.core.server.get_selection_ranges(self.file.id, &offsets);
        codec::selection_ranges(&self.file, ranges)
    }

    pub fn symbols(self, uri: &Uri) -> DocumentSymbolResponse {
        let items = self.workspace.core.server.get_document_symbols(self.file.id);
        codec::document_symbols(&self.file, uri, items, self.preferences.hierarchical_symbols)
    }

    pub fn hints(self, range: Range) -> Vec<InlayHint> {
        let start = self.offset(range.start);
        let end = self.offset(range.end);
        let hints = self.workspace.core.server.get_inlay_hints(self.file.id, start, end);
        codec::inlay_hints(&self.file, hints)
    }

    pub fn links(self) -> Vec<DocumentLink> {
        let links = self.workspace.core.server.get_document_links(self.file.id);
        links
            .into_iter()
            .filter_map(|link| {
                let target = self.workspace.database().get(&link.target.file).ok()?;
                let uri = self.workspace.uri(&target)?;
                let position = position_at_offset(&target, link.target.range.start);
                let uri = format!("{}#{},{}", uri.strip_fragment(), position.line + 1, position.character + 1)
                    .parse()
                    .ok()?;
                Some(codec::document_link(&self.file, link, uri))
            })
            .collect()
    }
}
