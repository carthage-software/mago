//! Link namespace imports to their resolved declarations.

use mago_database::file::FileId;

use crate::Server;
use crate::domain::DocumentLinkItem;
use crate::domain::Range;
use crate::domain::SymbolLocation;
use crate::semantic_index::SymbolKind;
use crate::symbol_resolution::constant_name_span;

impl Server {
    /// Links for class, function, and constant imports, including grouped imports.
    pub fn get_document_links(&mut self, file_id: FileId) -> Vec<DocumentLinkItem> {
        let Some(analysis) = self.file_analysis_for(file_id) else { return Vec::new() };
        let codebase = self.codebase();
        analysis
            .semantic()
            .imports
            .iter()
            .filter_map(|binding| {
                let span = match binding.kind {
                    SymbolKind::ClassLike => {
                        codebase.get_class_like(&binding.target).map(|meta| meta.name_span.unwrap_or(meta.span))
                    }
                    SymbolKind::Function => {
                        codebase.get_function(&binding.target).map(|meta| meta.name_span.unwrap_or(meta.span))
                    }
                    SymbolKind::Constant => {
                        codebase.get_constant(&binding.target).map(|meta| constant_name_span(self.database(), meta))
                    }
                    _ => None,
                }?;
                Some(DocumentLinkItem {
                    range: binding.target_range,
                    target: SymbolLocation {
                        file: span.file_id,
                        range: Range::new(span.start.offset, span.end.offset),
                    },
                    tooltip: String::from_utf8_lossy(&binding.target).into_owned(),
                })
            })
            .collect()
    }
}
