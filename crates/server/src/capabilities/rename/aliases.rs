//! Rename import bindings within their namespace block.

use foldhash::HashMap;
use mago_database::file::{File, FileId};

use crate::Server;
use crate::semantic_index::{SemanticIndex, SymbolKind};
use crate::{Range, RefusalReason, RenameRefusal, TextReplacement};

use super::{phpdoc, valid_name};

pub(super) fn at(index: &SemanticIndex, offset: u32) -> Option<(u32, Range)> {
    if let Some((id, range)) = index.imports.iter().enumerate().find_map(|(id, binding)| {
        binding.alias_range.filter(|range| range.contains(offset)).map(|range| (id as u32, range))
    }) {
        return Some((id, range));
    }
    let occurrence = index.at_offset(offset)?;
    Some((occurrence.import?, occurrence.alias_range.filter(|range| range.contains(offset))?))
}

pub(super) fn can_rename(
    index: &SemanticIndex,
    file: &File,
    import: u32,
    server: &Server,
) -> Result<(), RenameRefusal> {
    let binding = &index.imports[import as usize];
    phpdoc::alias_locations(server, file, binding, &binding.alias).map(drop)
}

pub(super) fn compute(
    index: &SemanticIndex,
    file: &File,
    import: u32,
    new_name: &str,
    server: &Server,
) -> Result<HashMap<FileId, Vec<TextReplacement>>, RenameRefusal> {
    let binding = &index.imports[import as usize];
    valid_name(file, binding.kind, new_name)?;
    if let Some(range) = native_conflict(index, file, import, new_name) {
        return Err(RenameRefusal::at(RefusalReason::AliasTaken, file.id, range));
    }
    let doc_locations = phpdoc::alias_locations(server, file, binding, new_name.as_bytes())?;
    let (range, replacement) = match binding.alias_range {
        Some(range) => (range, new_name.to_owned()),
        None => (Range::new(binding.target_range.end, binding.target_range.end), format!(" as {new_name}")),
    };
    let mut edits = vec![TextReplacement { range, new_text: replacement }];
    for occurrence in &index.occurrences {
        if occurrence.import == Some(import)
            && let Some(range) = occurrence.alias_range
        {
            edits.push(TextReplacement { range, new_text: new_name.to_owned() });
        }
    }
    edits.extend(doc_locations.into_iter().map(|range| TextReplacement { range, new_text: new_name.to_owned() }));
    Ok(HashMap::from_iter([(file.id, edits)]))
}

pub(super) fn native_conflict(index: &SemanticIndex, file: &File, import: u32, new_name: &str) -> Option<Range> {
    let binding = &index.imports[import as usize];
    let equal = |name: &[u8]| {
        if binding.kind == SymbolKind::Constant {
            name == new_name.as_bytes()
        } else {
            name.eq_ignore_ascii_case(new_name.as_bytes())
        }
    };
    index
        .imports
        .iter()
        .enumerate()
        .find(|(id, other)| {
            *id != import as usize
                && other.kind == binding.kind
                && other.namespace_range == binding.namespace_range
                && equal(&other.alias)
        })
        .map(|(_, other)| other.alias_range.unwrap_or(other.target_range))
        .or_else(|| {
            index
                .occurrences
                .iter()
                .find(|occurrence| {
                    if !binding.namespace_range.contains(occurrence.range.start) || occurrence.import == Some(import) {
                        return false;
                    }
                    let name = &file.contents[occurrence.range.start as usize..occurrence.range.end as usize];
                    let first = name.split(|byte| *byte == b'\\').next().unwrap_or(name);
                    let kind = index.symbols[occurrence.symbol as usize].kind;
                    (kind == binding.kind || (binding.kind == SymbolKind::ClassLike && first.len() < name.len()))
                        && equal(first)
                })
                .map(|occurrence| occurrence.range)
        })
}
