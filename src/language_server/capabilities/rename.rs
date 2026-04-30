//! `textDocument/rename` and `textDocument/prepareRename`.
//!
//! Symbol rename: collect every reference to the symbol under the cursor
//! (via [`crate::language_server::capabilities::references`]) and emit a
//! [`WorkspaceEdit`] that replaces each occurrence with the new name. We
//! don't try to fix up `use` statements or namespace prefixes yet, so
//! renames at the bare identifier level are safest for now.

use foldhash::HashMap;

use mago_database::file::File as MagoFile;
use mago_names::ResolvedNames;
use tower_lsp::lsp_types::PrepareRenameResponse;
use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::TextEdit;
use tower_lsp::lsp_types::Url;
use tower_lsp::lsp_types::WorkspaceEdit;

use crate::language_server::capabilities::lookup;
use crate::language_server::position::range_at_offsets;
use crate::language_server::state::WorkspaceState;

pub fn prepare(resolved: &ResolvedNames<'_>, file: &MagoFile, offset: u32) -> Option<PrepareRenameResponse> {
    if let Some((start, end, fqn, _)) = resolved.at_offset(offset) {
        let placeholder = fqn.rsplit('\\').next().unwrap_or(fqn).to_string();
        return Some(PrepareRenameResponse::RangeWithPlaceholder {
            range: range_at_offsets(file, start, end),
            placeholder,
        });
    }

    let var = lookup::variable_at_offset(file, offset)?;
    Some(PrepareRenameResponse::RangeWithPlaceholder {
        range: range_at_offsets(file, var.start, var.end),
        placeholder: var.name.to_string(),
    })
}

pub fn compute(
    workspace: &mut WorkspaceState,
    file: &MagoFile,
    offset: u32,
    new_name: String,
) -> Option<WorkspaceEdit> {
    if !is_valid_php_identifier(&new_name) {
        return None;
    }

    let locations = crate::language_server::capabilities::references::compute(workspace, file, offset, true);
    if locations.is_empty() {
        return None;
    }

    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::default();
    for loc in locations {
        changes
            .entry(loc.uri)
            .or_default()
            .push(TextEdit { range: Range { start: loc.range.start, end: loc.range.end }, new_text: new_name.clone() });
    }

    Some(WorkspaceEdit {
        changes: Some(changes.into_iter().collect()),
        document_changes: None,
        change_annotations: None,
    })
}

fn is_valid_php_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return false };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
