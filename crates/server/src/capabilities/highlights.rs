//! Highlights for one file's lexical binding or known member declaration.

use mago_database::DatabaseReader;
use mago_database::file::FileId;

use crate::Range;
use crate::Server;
use crate::call_analysis::ImplicitCalls;
use crate::semantic_index::SymbolKind;
use crate::symbol_resolution::{declared_member_at, members_at, methods_at};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HighlightKind {
    Text,
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Highlight {
    pub range: Range,
    pub kind: HighlightKind,
}

impl Server {
    /// Find highlights without reading or analyzing any other file.
    pub fn get_document_highlights(&mut self, file: FileId, offset: u32) -> Vec<Highlight> {
        let mut out = Vec::new();
        let mut targets: Vec<_> = declared_member_at(self.codebase(), file, offset).into_iter().collect();
        if let Some(editor) = self.editor_file(file) {
            targets.extend(
                methods_at(&editor, offset)
                    .iter()
                    .filter(|row| !row.dynamic && !row.magic && !row.uncertain)
                    .filter_map(|row| row.declaration),
            );
            targets.extend(
                members_at(&editor, offset)
                    .iter()
                    .filter(|row| !row.dynamic && !row.magic && !row.uncertain)
                    .filter_map(|row| row.declaration),
            );
            for row in &editor.methods {
                if !row.dynamic
                    && !row.magic
                    && !row.uncertain
                    && row.declaration.is_some_and(|span| targets.contains(&span))
                {
                    out.push(Highlight {
                        range: Range::new(row.selector_span.0, row.selector_span.1),
                        kind: HighlightKind::Read,
                    });
                }
            }
            for row in &editor.members {
                if !row.dynamic
                    && !row.magic
                    && !row.uncertain
                    && row.declaration.is_some_and(|span| targets.contains(&span))
                {
                    out.push(Highlight {
                        range: Range::new(row.selector_span.0, row.selector_span.1),
                        kind: if row.write { HighlightKind::Write } else { HighlightKind::Read },
                    });
                }
            }
            let implicit = ImplicitCalls::of(self.database(), &targets);
            if !implicit.is_empty()
                && let Some(analysis) = self.file_analysis_for(file)
                && let Ok(source) = self.database().get(&file)
            {
                out.extend(
                    implicit
                        .ranges(self.codebase(), &source, &analysis, Some(&editor), &targets, true)
                        .map(|range| Highlight { range, kind: HighlightKind::Read }),
                );
            }
        }
        if !targets.is_empty() {
            out.extend(targets.iter().filter(|span| span.file_id == file).map(|span| Highlight {
                range: Range::new(span.start.offset, span.end.offset),
                kind: HighlightKind::Text,
            }));
        } else if let Some(analysis) = self.file_analysis_for(file) {
            let index = analysis.semantic();
            let selected = index.at_offset(offset);
            let alias = index
                .imports
                .iter()
                .position(|binding| binding.alias_range.is_some_and(|range| range.contains(offset)))
                .map(|id| id as u32)
                .or_else(|| {
                    selected
                        .and_then(|entry| entry.alias_range.filter(|range| range.contains(offset)).and(entry.import))
                });
            if let Some(alias) = alias {
                let binding = &index.imports[alias as usize];
                out.push(Highlight {
                    range: binding.alias_range.unwrap_or_else(|| {
                        Range::new(binding.target_range.end - binding.alias.len() as u32, binding.target_range.end)
                    }),
                    kind: HighlightKind::Text,
                });
                out.extend(
                    index
                        .occurrences
                        .iter()
                        .filter(|entry| entry.import == Some(alias))
                        .filter_map(|entry| entry.alias_range)
                        .map(|range| Highlight { range, kind: HighlightKind::Text }),
                );
            } else if let Some(selected) = selected {
                let variable = index.symbols[selected.symbol as usize].kind == SymbolKind::Variable;
                out.extend(
                    index
                        .occurrences
                        .iter()
                        .filter(|entry| {
                            entry.symbol == selected.symbol && entry.alias_range != Some(entry.selection_range)
                        })
                        .map(|entry| Highlight {
                            range: if variable { entry.range } else { entry.selection_range },
                            kind: HighlightKind::Text,
                        }),
                );
            }
        }
        out.sort_unstable_by_key(|highlight| (highlight.range, highlight.kind));
        // A read/write access can have several analyzer rows; prefer its write marker.
        out.reverse();
        out.dedup_by_key(|highlight| highlight.range);
        out.reverse();
        out
    }
}
