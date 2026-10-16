//! Named object types from the last analysis and declaration metadata.

use mago_codex::ttype::atomic::TAtomic;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_word::Word;

use crate::Range;
use crate::Server;
use crate::SymbolLocation;
use crate::lookup;
use crate::semantic_index::SymbolKind;

impl Server {
    /// Class declarations for the selected expression's type, without running analysis.
    pub fn get_type_definitions(&mut self, file: FileId, offset: u32) -> Vec<SymbolLocation> {
        let Ok(source) = self.database().get(&file) else { return Vec::new() };
        if !self.lex(&source).iter().any(|token| {
            !lookup::is_trivia(token.kind)
                && token.start.offset <= offset
                && offset < token.start.offset + token.value.len() as u32
        }) {
            return Vec::new();
        }
        let Some(analysis) = self.file_analysis_for(file) else { return Vec::new() };
        let mut classes = Vec::new();
        if let Some(signature) = self.codebase().file_signatures.get(&file) {
            for node in &signature.ast_nodes {
                if node.is_function {
                    continue;
                }
                let Some(class) = self.codebase().get_class_like_by_name(node.name) else { continue };
                for property in class.properties.values() {
                    if property.name_span.is_some_and(|span| {
                        span.file_id == file && span.start.offset <= offset && offset < span.end.offset
                    }) && let Some(ty) = &property.type_metadata
                    {
                        for atomic in ty.type_union.types.iter() {
                            collect_classes(atomic, &mut classes);
                        }
                    }
                }
            }
        }
        if let Some(occurrence) = analysis.semantic().at_offset(offset) {
            let symbol = &analysis.semantic().symbols[occurrence.symbol as usize];
            if symbol.kind == SymbolKind::ClassLike
                && let Some(class) = self.codebase().get_class_like(&symbol.name)
            {
                classes.push(class.name);
            }
        }
        if classes.is_empty()
            && let Some(span) = analysis
                .expression_spans
                .iter()
                .filter(|(start, end)| *start <= offset && offset < *end)
                .min_by_key(|(start, end)| end - start)
            && let Some(editor) = self.editor_file(file)
            && let Ok(index) = editor.expression_classes.binary_search_by_key(span, |entry| entry.span)
        {
            classes.extend_from_slice(&editor.expression_classes[index].classes);
        }
        let mut locations: Vec<_> = classes
            .iter()
            .filter_map(|name| {
                let span = self.codebase().get_class_like_by_name(*name)?.name_span?;
                Some(SymbolLocation { file: span.file_id, range: Range::new(span.start.offset, span.end.offset) })
            })
            .collect();
        locations.sort_unstable_by_key(|location| (location.file, location.range));
        locations.dedup();
        locations
    }
}

fn collect_classes(atomic: &TAtomic, classes: &mut Vec<Word>) {
    classes.extend(atomic.get_all_object_names());
    if let TAtomic::GenericParameter(parameter) = atomic {
        for atomic in parameter.constraint.types.iter() {
            collect_classes(atomic, classes);
        }
    }
}
