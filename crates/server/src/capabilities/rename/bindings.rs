//! Check local bindings and captures before gathering rename locations.

use foldhash::HashSet;
use mago_database::file::FileId;

use crate::semantic_index::{BindingKind, Occurrence, ScopeKind, SemanticIndex, SymbolKind};
use crate::{RefusalReason, RenameRefusal, SymbolLocation};

use super::{NO_SYMBOL, STALE, bound_elsewhere};

pub(super) fn prepare(file: FileId, index: &SemanticIndex, selected: &Occurrence) -> Result<(), RenameRefusal> {
    ensure_explicit(file, index, selected)?;
    ensure_unbound(index, file, selected.symbol, None)
}

pub(super) fn locations(
    file: FileId,
    index: &SemanticIndex,
    selected: &Occurrence,
    new_name: &str,
) -> Result<Vec<SymbolLocation>, RenameRefusal> {
    let symbol = selected.symbol;
    ensure_explicit(file, index, selected)?;
    ensure_unbound(index, file, symbol, Some(new_name))?;
    let original = &index.symbols[symbol as usize];
    let mut occurrences = index.occurrences.iter().filter(|occurrence| occurrence.symbol == symbol).peekable();
    let first = occurrences.peek().ok_or(NO_SYMBOL)?;
    let mut renamed = original.clone();
    renamed.name = new_name.as_bytes().into();
    if let Some(range) = bound_elsewhere(index, &renamed, original, first.range) {
        return Err(RenameRefusal::at(RefusalReason::BindingTaken, file, range));
    }
    Ok(occurrences.map(|occurrence| SymbolLocation { file, range: occurrence.selection_range }).collect())
}

fn ensure_explicit(file: FileId, index: &SemanticIndex, selected: &Occurrence) -> Result<(), RenameRefusal> {
    let symbol = selected.symbol;
    if index.symbols[symbol as usize].binding == Some(BindingKind::Parameter)
        && !index.occurrences.iter().any(|occurrence| occurrence.symbol == symbol && occurrence.declaration)
    {
        return Err(RenameRefusal::at(RefusalReason::ImplicitParameter, file, selected.range));
    }
    Ok(())
}

pub(super) fn ensure_unbound(
    index: &SemanticIndex,
    file: FileId,
    symbol: u32,
    new_name: Option<&str>,
) -> Result<(), RenameRefusal> {
    let mut scopes = HashSet::from_iter([index.symbols[symbol as usize].scope.ok_or(STALE)?]);
    for occurrence in index.occurrences.iter().filter(|occurrence| occurrence.symbol == symbol) {
        let mut scope = scope_at(index, occurrence.range.start).ok_or(STALE)?;
        loop {
            scopes.insert(scope);
            let current = &index.scopes[scope as usize];
            if current.kind != ScopeKind::ArrowFunction {
                break;
            }
            // Each arrow captures through its parent, even if only a nested arrow uses the name.
            scope = current.parent.ok_or(STALE)?;
        }
    }
    let name = index.symbols[symbol as usize].name.as_ref();
    if let Some(runtime) = index.runtime_variables.iter().find(|runtime| {
        scopes.contains(&runtime.scope)
            && runtime
                .name
                .as_deref()
                .is_none_or(|used| used == name || new_name.is_some_and(|new| new.as_bytes() == used))
    }) {
        return Err(RenameRefusal::at(RefusalReason::RuntimeVariables, file, runtime.range));
    }
    let Some(new_name) = new_name else { return Ok(()) };
    for occurrence in &index.occurrences {
        let other = &index.symbols[occurrence.symbol as usize];
        if occurrence.symbol != symbol
            && other.kind == SymbolKind::Variable
            && other.name.as_ref() == new_name.as_bytes()
            && (scopes.contains(&other.scope.ok_or(STALE)?)
                || scopes.contains(&scope_at(index, occurrence.range.start).ok_or(STALE)?))
        {
            return Err(RenameRefusal::at(RefusalReason::BindingTaken, file, occurrence.range));
        }
    }
    Ok(())
}

fn scope_at(index: &SemanticIndex, offset: u32) -> Option<u32> {
    index.scopes.iter().rposition(|scope| scope.range.contains(offset)).map(|scope| scope as u32)
}
