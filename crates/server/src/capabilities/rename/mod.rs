//! Rename PHP bindings and members with byte-based edits.

use std::sync::Arc;

use foldhash::HashMap;
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId};
use mago_span::Span;

use crate::semantic_index::{BindingKind, Occurrence, ScopeKind, SemanticIndex, SymbolKey, SymbolKind};
use crate::{Range, RefusalReason, RenameRefusal, Server, SymbolLocation, TextReplacement};

mod aliases;
mod bindings;
mod globals;
mod names;
mod parameters;
mod phpdoc;

#[cfg(test)]
mod tests;

pub(crate) use names::invalid;
use names::{identifier, valid_name};
pub(super) use parameters::extend_properties;

pub(crate) const STALE: RenameRefusal = RenameRefusal::new(RefusalReason::Stale);
pub(crate) const NO_SYMBOL: RenameRefusal = RenameRefusal::new(RefusalReason::NoSymbol);

impl Server {
    /// Return the selected name's byte range when its binding can be renamed.
    ///
    /// # Errors
    ///
    /// Returns a [`RenameRefusal`] with the check that declined the rename.
    pub fn get_prepare_rename(&mut self, file_id: FileId, offset: u32) -> Result<Range, RenameRefusal> {
        let file = self.rename_source(file_id, offset)?;
        let analysis = self.file_analysis_for(file_id).ok_or(STALE)?;
        let index = analysis.semantic();
        let range = if let Some((import, range)) = aliases::at(index, offset) {
            aliases::can_rename(index, &file, import, self)?;
            range
        } else if let Some(occurrence) = index.at_offset(offset) {
            let symbol = &index.symbols[occurrence.symbol as usize];
            if signature_binding(index, symbol) {
                return Ok(parameters::rename(self, file_id, offset, None)?.ok_or(NO_SYMBOL)?.range);
            }
            selectable(file_id, symbol.kind, occurrence, offset)?;
            if symbol.kind == SymbolKind::Variable {
                bindings::prepare(file_id, index, occurrence)?;
            } else {
                globals::prepare(self, &file, index, occurrence)?;
            }
            occurrence.selection_range
        } else {
            match parameters::rename(self, file_id, offset, None)? {
                Some(rename) => rename.range,
                None => self.get_member_rename(file_id, offset, None)?.range,
            }
        };
        Ok(range)
    }

    /// Return all checked replacements, grouped by source file.
    ///
    /// # Errors
    ///
    /// Returns a [`RenameRefusal`] with the check that declined the rename.
    pub fn get_rename(
        &mut self,
        file_id: FileId,
        offset: u32,
        new_name: &str,
    ) -> Result<HashMap<FileId, Vec<TextReplacement>>, RenameRefusal> {
        let edits = self.rename_edits(file_id, offset, new_name)?;
        if let Some((file, replacements)) = edits.iter().find(|(file, _)| self.editable_file(**file).is_none()) {
            let range = replacements.first().map_or(Range::new(0, 0), |replacement| replacement.range);
            return Err(RenameRefusal::at(RefusalReason::ReadOnly, *file, range));
        }
        Ok(edits)
    }

    pub(crate) fn rename_source(&self, file: FileId, offset: u32) -> Result<Arc<File>, RenameRefusal> {
        let source = self.database().get(&file).map_err(|_| NO_SYMBOL)?;
        if source.file_type.is_builtin() || source.file_type.is_external() {
            return Err(RenameRefusal::at(RefusalReason::ReadOnly, file, Range::new(offset, offset)));
        }
        Ok(source)
    }

    fn rename_edits(
        &mut self,
        file_id: FileId,
        offset: u32,
        new_name: &str,
    ) -> Result<HashMap<FileId, Vec<TextReplacement>>, RenameRefusal> {
        let file = self.rename_source(file_id, offset)?;
        let analysis = self.file_analysis_for(file_id).ok_or(STALE)?;
        let index = analysis.semantic();
        if let Some((import, _)) = aliases::at(index, offset) {
            return aliases::compute(index, &file, import, new_name, self);
        }
        let Some(selected) = index.at_offset(offset) else {
            identifier(new_name)?;
            let rename = match parameters::rename(self, file_id, offset, Some(new_name))? {
                Some(rename) => rename,
                None => self.get_member_rename(file_id, offset, Some(new_name.as_bytes()))?,
            };
            return Ok(edits_for(rename.locations, new_name));
        };
        let symbol = &index.symbols[selected.symbol as usize];
        if signature_binding(index, symbol) {
            let rename = parameters::rename(self, file_id, offset, Some(new_name))?.ok_or(NO_SYMBOL)?;
            return Ok(edits_for(rename.locations, new_name));
        }
        selectable(file_id, symbol.kind, selected, offset)?;
        valid_name(&file, symbol.kind, new_name)?;
        let locations = if symbol.kind == SymbolKind::Variable {
            bindings::locations(file_id, index, selected, new_name)?
        } else {
            globals::locations(self, &file, offset, index, selected, new_name)?
        };
        Ok(edits_for(locations, new_name))
    }
}

pub(crate) fn declared_outside(file: &File, span: Span) -> RenameRefusal {
    let reason = if file.file_type.is_builtin() { RefusalReason::Builtin } else { RefusalReason::External };
    RenameRefusal::at(reason, file.id, Range::new(span.start.offset, span.end.offset))
}

fn signature_binding(index: &SemanticIndex, symbol: &SymbolKey) -> bool {
    symbol.binding.is_some_and(|binding| binding != BindingKind::Local)
        && symbol.scope.is_none_or(|scope| index.scopes[scope as usize].kind != ScopeKind::PropertyHook)
}

fn selectable(file: FileId, kind: SymbolKind, occurrence: &Occurrence, offset: u32) -> Result<(), RenameRefusal> {
    if kind == SymbolKind::Namespace {
        return Err(RenameRefusal::at(RefusalReason::Namespace { import: false }, file, occurrence.range));
    }
    if kind != SymbolKind::Variable && !occurrence.selection_range.contains(offset) {
        return Err(RenameRefusal::at(RefusalReason::NamespaceSegment, file, occurrence.range));
    }
    Ok(())
}

fn bound_elsewhere(index: &SemanticIndex, renamed: &SymbolKey, original: &SymbolKey, fallback: Range) -> Option<Range> {
    let symbol = index.symbols.iter().position(|candidate| candidate == renamed && candidate != original)?;
    Some(
        index
            .occurrences
            .iter()
            .find(|occurrence| occurrence.symbol as usize == symbol)
            .map_or(fallback, |occurrence| occurrence.range),
    )
}

fn edits_for(locations: Vec<SymbolLocation>, new_name: &str) -> HashMap<FileId, Vec<TextReplacement>> {
    let mut edits: HashMap<FileId, Vec<TextReplacement>> = HashMap::default();
    for location in locations {
        edits
            .entry(location.file)
            .or_default()
            .push(TextReplacement { range: location.range, new_text: new_name.to_owned() });
    }
    edits
}
