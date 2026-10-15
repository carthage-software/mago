//! `get_definition`: resolve the identifier under the cursor to its
//! fully-qualified name and look up that symbol's declaration span.

use mago_database::DatabaseReader;
use mago_database::file::FileId;

use crate::Server;
use crate::domain::Range;
use crate::domain::SymbolLocation;
use crate::semantic_index::SymbolKind;
use crate::symbol_resolution::{declared_member_at, members_at, methods_at, resolved_symbol_name, same_name};

impl Server {
    /// The declaration location of the symbol whose identifier covers `offset`
    /// in `file_id`, or `None` if the offset isn't on a resolvable name.
    pub fn get_definition(&mut self, file_id: FileId, offset: u32) -> Option<SymbolLocation> {
        self.get_definitions(file_id, offset).into_iter().next()
    }

    /// All known declarations, including distinct targets of a union receiver.
    pub fn get_definitions(&mut self, file_id: FileId, offset: u32) -> Vec<SymbolLocation> {
        if let Some(span) = declared_member_at(self.codebase(), file_id, offset) {
            return vec![SymbolLocation { file: span.file_id, range: Range::new(span.start.offset, span.end.offset) }];
        }
        if let Some(editor) = self.editor_file(file_id) {
            let mut locations: Vec<_> = methods_at(&editor, offset)
                .iter()
                .filter_map(|method| method.declaration)
                .map(|span| SymbolLocation {
                    file: span.file_id,
                    range: Range::new(span.start.offset, span.end.offset),
                })
                .collect();
            locations.extend(members_at(&editor, offset).iter().filter_map(|member| {
                member.declaration.map(|span| SymbolLocation {
                    file: span.file_id,
                    range: Range::new(span.start.offset, span.end.offset),
                })
            }));
            locations.sort_unstable_by_key(|location| (location.file, location.range.start, location.range.end));
            locations.dedup();
            if !locations.is_empty() {
                return locations;
            }
        }
        let Some(analysis) = self.file_analysis_for(file_id) else { return Vec::new() };
        let index = analysis.semantic();
        let Some(occurrence) = index.at_offset(offset) else { return Vec::new() };
        let symbol = &index.symbols[occurrence.symbol as usize];
        if symbol.kind == SymbolKind::Variable {
            return index
                .occurrences
                .iter()
                .filter(|entry| entry.symbol == occurrence.symbol && entry.declaration)
                .map(|entry| SymbolLocation { file: file_id, range: entry.range })
                .collect();
        }
        let Ok(file) = self.database().get(&file_id) else { return Vec::new() };
        let name = resolved_symbol_name(self.codebase(), &file.contents, index, occurrence);
        let span = match symbol.kind {
            SymbolKind::ClassLike => {
                self.codebase().get_class_like(name).map(|meta| meta.name_span.unwrap_or(meta.span))
            }
            SymbolKind::Function => self.codebase().get_function(name).map(|meta| meta.name_span.unwrap_or(meta.span)),
            SymbolKind::Constant => self.codebase().get_constant(name).map(|meta| meta.span),
            _ => None,
        };
        let Some(span) = span else { return Vec::new() };
        let target = self.file_analysis_for(span.file_id);
        let range = target
            .as_ref()
            .and_then(|target| {
                target.semantic().occurrences.iter().find(|entry| {
                    let candidate = &target.semantic().symbols[entry.symbol as usize];
                    entry.declaration && candidate.kind == symbol.kind && same_name(symbol.kind, &candidate.name, name)
                })
            })
            .map_or(Range::new(span.start.offset, span.end.offset), |entry| entry.selection_range);
        vec![SymbolLocation { file: span.file_id, range }]
    }
}
