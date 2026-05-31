//! `textDocument/rename` and `textDocument/prepareRename`.
//!
//! Symbol rename: collect every reference to the symbol under the cursor
//! (via [`crate::language_server::capabilities::references`]) and emit a
//! [`WorkspaceEdit`] that replaces each occurrence with the new name. We
//! don't try to fix up `use` statements or namespace prefixes yet, so
//! renames at the bare identifier level are safest for now.

use foldhash::HashMap;
use tower_lsp_server::ls_types::PrepareRenameResponse;
use tower_lsp_server::ls_types::TextEdit;
use tower_lsp_server::ls_types::Uri;
use tower_lsp_server::ls_types::WorkspaceEdit;

use mago_database::file::File as MagoFile;
use mago_names::ResolvedNames;
use mago_server::lookup;

use crate::language_server::codec;
use crate::language_server::position::range_at_offsets;
use crate::language_server::state::WorkspaceState;

pub fn prepare(resolved: &ResolvedNames<'_>, file: &MagoFile, offset: u32) -> Option<PrepareRenameResponse> {
    if let Some((start, end, fqn, _)) = resolved.at_offset(offset) {
        let local = match memchr::memrchr(b'\\', fqn) {
            Some(i) => &fqn[i + 1..],
            None => fqn,
        };
        let placeholder = String::from_utf8_lossy(local).into_owned();
        return Some(PrepareRenameResponse::RangeWithPlaceholder {
            range: range_at_offsets(file, start, end),
            placeholder,
        });
    }

    let var = lookup::variable_at_offset(file, offset)?;
    Some(PrepareRenameResponse::RangeWithPlaceholder {
        range: range_at_offsets(file, var.start, var.end),
        placeholder: String::from_utf8_lossy(var.name).into_owned(),
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

    let references = workspace.server.get_references(file.id, offset, true);
    if references.is_empty() {
        return None;
    }

    let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::default();
    for reference in references {
        if let Some(location) = codec::location(workspace.database(), &reference) {
            changes
                .entry(location.uri)
                .or_default()
                .push(TextEdit { range: location.range, new_text: new_name.clone() });
        }
    }

    if changes.is_empty() {
        return None;
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
