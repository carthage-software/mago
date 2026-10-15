//! Typed references and lexical local-variable bindings.

use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_database::file::FileType;

use crate::Range;
use crate::Server;
use crate::domain::SymbolLocation;
use crate::semantic_index::SymbolKind;
use crate::symbol_resolution::{
    declared_member_at, may_refer_to, members_at, methods_at, resolved_symbol_name, same_name,
};

impl Server {
    /// Find references with the same symbol kind and binding as the selected name.
    pub fn get_references(&mut self, file_id: FileId, offset: u32, include_declaration: bool) -> Vec<SymbolLocation> {
        let mut targets = Vec::new();
        if let Some(span) = declared_member_at(self.codebase(), file_id, offset) {
            targets.push(span);
        }
        if let Some(editor) = self.editor_file(file_id) {
            targets.extend(methods_at(&editor, offset).iter().filter_map(|method| method.declaration));
            targets.extend(members_at(&editor, offset).iter().filter_map(|member| member.declaration));
        }
        if !targets.is_empty() {
            let mut locations = Vec::new();
            if include_declaration {
                locations.extend(targets.iter().map(|span| SymbolLocation {
                    file: span.file_id,
                    range: Range::new(span.start.offset, span.end.offset),
                }));
            }
            for file in self.database().files().filter(|file| file.file_type == FileType::Host) {
                let Some(editor) = self.editor_file(file.id) else { continue };
                locations.extend(
                    editor
                        .methods
                        .iter()
                        .filter(|row| row.declaration.is_some_and(|span| targets.contains(&span)))
                        .map(|row| SymbolLocation {
                            file: file.id,
                            range: Range::new(row.selector_span.0, row.selector_span.1),
                        }),
                );
                locations.extend(
                    editor
                        .members
                        .iter()
                        .filter(|row| row.declaration.is_some_and(|span| targets.contains(&span)))
                        .map(|row| SymbolLocation {
                            file: file.id,
                            range: Range::new(row.selector_span.0, row.selector_span.1),
                        }),
                );
            }
            locations.sort_unstable_by_key(|location| (location.file, location.range.start, location.range.end));
            locations.dedup();
            return locations;
        }
        let Some(analysis) = self.file_analysis_for(file_id) else { return Vec::new() };
        let index = analysis.semantic();
        let alias = index
            .imports
            .iter()
            .position(|binding| binding.alias_range.is_some_and(|range| range.contains(offset)))
            .map(|id| id as u32)
            .or_else(|| {
                let occurrence = index.at_offset(offset)?;
                occurrence
                    .alias_range
                    .filter(|range| range.contains(offset) && *range != occurrence.selection_range)?;
                occurrence.import
            });
        if let Some(id) = alias {
            let binding = &index.imports[id as usize];
            let mut locations = Vec::new();
            if include_declaration {
                locations.push(SymbolLocation {
                    file: file_id,
                    range: binding.alias_range.unwrap_or_else(|| {
                        Range::new(binding.target_range.end - binding.alias.len() as u32, binding.target_range.end)
                    }),
                });
            }
            locations.extend(
                index
                    .occurrences
                    .iter()
                    .filter(|entry| entry.import == Some(id))
                    .filter_map(|entry| entry.alias_range.map(|range| SymbolLocation { file: file_id, range })),
            );
            locations.sort_unstable_by_key(|location| location.range.start);
            return locations;
        }
        let Some(occurrence) = index.at_offset(offset) else { return Vec::new() };
        let symbol = &index.symbols[occurrence.symbol as usize];
        if symbol.kind == SymbolKind::Variable {
            return index
                .occurrences
                .iter()
                .filter(|entry| entry.symbol == occurrence.symbol && (include_declaration || !entry.declaration))
                .map(|entry| SymbolLocation { file: file_id, range: entry.range })
                .collect();
        }
        let Ok(file) = self.database().get(&file_id) else { return Vec::new() };
        let name = resolved_symbol_name(self.codebase(), &file.contents, index, occurrence);

        let candidates: Vec<_> =
            self.database().files().filter(|file| file.file_type == FileType::Host).map(|file| file.id).collect();
        let mut locations = Vec::new();
        for id in candidates {
            let Some(analysis) = self.file_analysis_for(id) else { continue };
            let Ok(file) = self.database().get(&id) else { continue };
            let index = analysis.semantic();
            let candidate = |candidate: &crate::semantic_index::SymbolKey| {
                candidate.kind == symbol.kind && may_refer_to(symbol.kind, &candidate.name, name)
            };
            if !index.symbols.iter().any(candidate) {
                continue;
            }
            locations.extend(
                index
                    .occurrences
                    .iter()
                    .filter(|entry| {
                        candidate(&index.symbols[entry.symbol as usize])
                            && (include_declaration || !entry.declaration)
                            && same_name(
                                symbol.kind,
                                resolved_symbol_name(self.codebase(), &file.contents, index, entry),
                                name,
                            )
                    })
                    .map(|entry| SymbolLocation { file: id, range: entry.range }),
            );
        }
        locations.sort_unstable_by_key(|location| (location.file, location.range.start, location.range.end));
        locations
    }
}
