//! Check global declarations, namespace fallback, and imports before gathering rename locations.

use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_database::file::FileType;
use mago_span::Span;
use mago_word::ascii_lowercase_word;

use crate::resolved_symbol_name;
use crate::semantic_index::{Occurrence, SemanticIndex, SymbolKind};
use crate::symbol_resolution::is_define_call;
use crate::{Range, RefusalReason, RenameRefusal, Server, SymbolLocation};

use super::{STALE, aliases, bound_elsewhere, declared_outside, phpdoc};

pub(super) fn prepare(
    server: &Server,
    file: &MagoFile,
    index: &SemanticIndex,
    occurrence: &Occurrence,
) -> Result<(), RenameRefusal> {
    host_declaration(server, file, index, occurrence)?;
    if index.symbols[occurrence.symbol as usize].kind == SymbolKind::Function {
        let name = resolved_symbol_name(server.codebase(), &file.contents, index, occurrence);
        string_reference(server.callable_function_reference(&[name])?)?;
    }
    Ok(())
}

pub(super) fn locations(
    server: &mut Server,
    file: &MagoFile,
    offset: u32,
    index: &SemanticIndex,
    selected: &Occurrence,
    new_name: &str,
) -> Result<Vec<SymbolLocation>, RenameRefusal> {
    host_declaration(server, file, index, selected)?;
    let symbol = &index.symbols[selected.symbol as usize];
    let mut target = symbol.clone();
    target.name = resolved_symbol_name(server.codebase(), &file.contents, index, selected).into();
    if target.kind == SymbolKind::Function {
        target.name.make_ascii_lowercase();
    }
    let symbol = &target;
    let mut renamed = symbol.clone();
    renamed.name = renamed_name(symbol.kind, &symbol.name, new_name);
    if &renamed != symbol
        && let Some(span) = declaration(server, symbol.kind, &renamed.name)
    {
        let source = server.database().get(&span.file_id).map_err(|_| STALE)?;
        let span = named(&source, symbol.kind, &renamed.name, span);
        return Err(RenameRefusal::at(RefusalReason::NameTaken, span.file_id, range(span)));
    }
    if symbol.kind == SymbolKind::ClassLike && &renamed != symbol {
        string_reference(server.callable_class_reference(&[&symbol.name, &renamed.name])?)?;
    }
    if symbol.kind == SymbolKind::Function && &renamed != symbol {
        string_reference(server.callable_function_reference(&[&symbol.name, &renamed.name])?)?;
    }
    if matches!(symbol.kind, SymbolKind::Function | SymbolKind::Constant) && renamed.name.contains(&b'\\') {
        let files: Vec<_> = server.database().files().filter(|file| file.file_type == FileType::Host).collect();
        for file in files {
            let analysis = server.file_analysis_for(file.id).ok_or(STALE)?;
            let index = analysis.semantic();
            if let Some(occurrence) = index.occurrences.iter().find(|occurrence| {
                let candidate = &index.symbols[occurrence.symbol as usize];
                candidate == &renamed
                    && !same_name(
                        symbol.kind,
                        &candidate.name,
                        resolved_symbol_name(server.codebase(), &file.contents, index, occurrence),
                    )
            }) {
                return Err(RenameRefusal::at(RefusalReason::Capture, file.id, occurrence.range));
            }
        }
    }
    let references = server.get_references(file.id, offset, true);
    let mut locations = Vec::new();
    let mut has_declaration = false;
    for reference in references {
        let reference_file = server.database().get(&reference.file).map_err(|_| STALE)?;
        let analysis = server.file_analysis_for(reference.file).ok_or(STALE)?;
        let index = analysis.semantic();
        let occurrence = index.at_offset(reference.range.start).ok_or(STALE)?;
        let indexed = &index.symbols[occurrence.symbol as usize];
        if matches!(symbol.kind, SymbolKind::Function | SymbolKind::Constant)
            && !same_name(
                symbol.kind,
                &indexed.name,
                resolved_symbol_name(server.codebase(), &reference_file.contents, index, occurrence),
            )
            && declaration(server, symbol.kind, &renamed_name(symbol.kind, &indexed.name, new_name)).is_some()
        {
            return Err(RenameRefusal::at(RefusalReason::Capture, reference.file, occurrence.range));
        }
        has_declaration |= occurrence.declaration;
        if let Some(range) = bound_elsewhere(index, &renamed, symbol, occurrence.range) {
            return Err(RenameRefusal::at(RefusalReason::Capture, reference.file, range));
        }
        if let Some(id) = occurrence.import {
            let binding = &index.imports[id as usize];
            if binding.target_range == occurrence.range
                && index.occurrences.iter().any(|other| {
                    other.import == Some(id) && other.alias_range.is_some_and(|range| range != other.selection_range)
                })
            {
                return Err(RenameRefusal::at(RefusalReason::ImportPrefix, reference.file, binding.target_range));
            }
            if occurrence.alias_range == Some(occurrence.selection_range) && binding.alias_range.is_some() {
                continue;
            }
            if binding.alias_range.is_none()
                && binding.target_range == occurrence.range
                && let Some(range) = aliases::native_conflict(index, &reference_file, id, new_name)
            {
                return Err(RenameRefusal::at(RefusalReason::AliasTaken, reference.file, range));
            }
        }
        if occurrence.import.is_none()
            && (occurrence.range == occurrence.selection_range || occurrence.declaration)
            && let Some(binding) = index.imports.iter().find(|binding| {
                binding.kind == symbol.kind
                    && binding.namespace_range.contains(occurrence.range.start)
                    && if symbol.kind == SymbolKind::Constant {
                        binding.alias.as_ref() == new_name.as_bytes()
                    } else {
                        binding.alias.eq_ignore_ascii_case(new_name.as_bytes())
                    }
            })
        {
            return Err(RenameRefusal::at(
                RefusalReason::AliasTaken,
                reference.file,
                binding.alias_range.unwrap_or(binding.target_range),
            ));
        }
        locations.push(SymbolLocation { file: reference.file, range: occurrence.selection_range });
    }
    if locations.is_empty() || !has_declaration {
        return Err(STALE);
    }
    if symbol.kind == SymbolKind::ClassLike {
        locations.extend(phpdoc::class_locations(server, &symbol.name, &renamed.name, new_name)?);
    }
    Ok(locations)
}

fn string_reference(found: Option<(SymbolLocation, usize)>) -> Result<(), RenameRefusal> {
    match found {
        Some((location, name)) => Err(RenameRefusal {
            reason: RefusalReason::StringReference { proposed: name > 0 },
            location: Some(location),
        }),
        None => Ok(()),
    }
}

fn same_name(kind: SymbolKind, left: &[u8], right: &[u8]) -> bool {
    if kind == SymbolKind::Constant { left == right } else { left.eq_ignore_ascii_case(right) }
}

fn renamed_name(kind: SymbolKind, name: &[u8], new_name: &str) -> Box<[u8]> {
    let prefix = name.iter().rposition(|byte| *byte == b'\\').map_or(0, |position| position + 1);
    let mut renamed = name[..prefix].to_vec();
    renamed.extend_from_slice(new_name.as_bytes());
    if matches!(kind, SymbolKind::ClassLike | SymbolKind::Function) {
        renamed.make_ascii_lowercase();
    }
    renamed.into_boxed_slice()
}

fn declaration(server: &Server, kind: SymbolKind, name: &[u8]) -> Option<Span> {
    let codebase = server.codebase();
    match kind {
        SymbolKind::ClassLike => {
            codebase.get_class_like(name).map(|metadata| metadata.name_span.unwrap_or(metadata.span))
        }
        SymbolKind::Function => codebase.get_function(name).map(|metadata| metadata.name_span.unwrap_or(metadata.span)),
        SymbolKind::Constant => codebase.get_constant(name).map(|metadata| metadata.span),
        _ => None,
    }
}

fn host_declaration(
    server: &Server,
    file: &MagoFile,
    index: &SemanticIndex,
    occurrence: &Occurrence,
) -> Result<(), RenameRefusal> {
    let kind = index.symbols[occurrence.symbol as usize].kind;
    let name = resolved_symbol_name(server.codebase(), &file.contents, index, occurrence);
    if kind == SymbolKind::ClassLike
        && let Some(span) = alias_declaration(server, name)
    {
        return Err(RenameRefusal::at(RefusalReason::RuntimeDeclaration, span.file_id, range(span)));
    }
    let Some(span) = declaration(server, kind, name) else {
        return Err(undeclared(server, file, index, occurrence, kind));
    };
    let source = server.database().get(&span.file_id).map_err(|_| STALE)?;
    let named = named(&source, kind, name, span);
    if source.file_type != FileType::Host {
        return Err(declared_outside(&source, named));
    }
    if kind == SymbolKind::Constant && named == span {
        return Err(RenameRefusal::at(RefusalReason::RuntimeDeclaration, span.file_id, range(span)));
    }
    Ok(())
}

fn alias_declaration(server: &Server, name: &[u8]) -> Option<Span> {
    let codebase = server.codebase();
    let alias = ascii_lowercase_word(name);
    if !codebase.class_like_aliases.contains_key(&alias) {
        return None;
    }
    codebase.class_like_alias_declarations().find_map(|(declared, _, span)| (declared == alias).then_some(span))
}

fn named(file: &MagoFile, kind: SymbolKind, name: &[u8], span: Span) -> Span {
    let short = name.rsplit(|byte| *byte == b'\\').next().unwrap_or(name);
    if kind == SymbolKind::Constant
        && file
            .contents
            .get(span.start.offset as usize..)
            .is_some_and(|rest| !is_define_call(rest) && rest.starts_with(short))
    {
        span.subspan(0, short.len() as u32)
    } else {
        span
    }
}

fn undeclared(
    server: &Server,
    file: &MagoFile,
    index: &SemanticIndex,
    occurrence: &Occurrence,
    kind: SymbolKind,
) -> RenameRefusal {
    if kind == SymbolKind::ClassLike
        && let Some(id) = occurrence.import
        && let Some(binding) = index.imports.get(id as usize)
        && binding.target_range == occurrence.range
        && (server.codebase().namespace_exists(&binding.target)
            || index.occurrences.iter().any(|other| {
                other.import == Some(id) && other.alias_range.is_some_and(|range| range != other.selection_range)
            }))
    {
        return RenameRefusal::at(RefusalReason::Namespace { import: true }, file.id, binding.target_range);
    }
    RenameRefusal::at(RefusalReason::Undeclared, file.id, occurrence.range)
}

const fn range(span: Span) -> Range {
    Range::new(span.start.offset, span.end.offset)
}
