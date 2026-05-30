//! `textDocument/references`.
//!
//! Resolves the symbol under the cursor to its fully-qualified name, then asks
//! each host file's [`ResolvedNames`](mago_names::ResolvedNames) for matching
//! references via [`references_to`](mago_names::ResolvedNames::references_to).
//! Matching is on resolved FQNs, so aliased
//! imports are handled correctly:
//!
//! ```php
//! namespace Foo;
//! use Bar as Qux;
//! $x = Qux\G;        // resolves to Bar\G
//! ```
//!
//! A coarse byte-level filter skips files that can't mention the local name,
//! avoiding parse + resolve on irrelevant files.
//!
//! Variables (`$foo`) aren't tracked by name resolution; they fall through to a
//! same-file token scan. Only `FileType::Host` is searched; prelude / vendored
//! sources aren't user-facing navigation targets.

use std::sync::Arc;

use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_database::file::FileType;
use mago_syntax::token::TokenKind;
use tower_lsp_server::ls_types::Location;
use tower_lsp_server::ls_types::Uri;

use crate::language_server::capabilities::lookup;
use crate::language_server::position::range_at_offsets;
use crate::language_server::state::WorkspaceState;

pub fn compute(
    workspace: &mut WorkspaceState,
    file: &MagoFile,
    offset: u32,
    include_declaration: bool,
) -> Vec<Location> {
    if let Some(var) = lookup::variable_at_offset(file, offset) {
        return same_file_variable_locations(file, var.raw);
    }

    let Some(cursor_analysis) = workspace.file_analysis_for(file.id) else { return Vec::new() };
    let Some((_, _, target_fqn, _)) = cursor_analysis.resolved().at_offset(offset) else { return Vec::new() };
    let target_fqn = target_fqn.to_vec();

    let local_lower = local_name(&target_fqn).to_ascii_lowercase();
    let declaration = if include_declaration { None } else { workspace.service.codebase().span_of(&target_fqn) };

    let candidates: Vec<Arc<MagoFile>> = workspace
        .database
        .files()
        .filter(|f| matches!(f.file_type, FileType::Host))
        .filter(|f| might_contain(f.contents.as_ref(), &local_lower))
        .collect();

    let mut out = Vec::new();
    for arc_file in candidates {
        let Some(analysis) = workspace.file_analysis_for(arc_file.id) else { continue };
        let Some(path) = arc_file.path.as_ref() else { continue };
        let Some(url) = Uri::from_file_path(path) else { continue };

        let exclude = declaration.filter(|d| d.file_id == arc_file.id).map(|d| d.start.offset);
        for (start, end) in analysis.resolved().references_to(&target_fqn, exclude) {
            out.push(Location { uri: url.clone(), range: range_at_offsets(&arc_file, start, end) });
        }
    }

    out
}

fn local_name(fqcn: &[u8]) -> &[u8] {
    match memchr::memrchr(b'\\', fqcn) {
        Some(i) => &fqcn[i + 1..],
        None => fqcn,
    }
}

/// Coarse case-insensitive containment pre-filter: does `haystack` possibly
/// mention `needle`? Lets the caller skip files that can't contain the symbol.
fn might_contain(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }

    if haystack.len() < needle.len() {
        return false;
    }

    let last = haystack.len() - needle.len();
    (0..=last).any(|i| haystack[i..i + needle.len()].iter().zip(needle).all(|(a, b)| a.eq_ignore_ascii_case(b)))
}

fn same_file_variable_locations(file: &MagoFile, raw: &[u8]) -> Vec<Location> {
    let Some(path) = file.path.as_ref() else {
        return Vec::new();
    };

    let Some(url) = Uri::from_file_path(path) else {
        return Vec::new();
    };

    lookup::lex(file)
        .into_iter()
        .filter(|t| matches!(t.kind, TokenKind::Variable) && t.value == raw)
        .map(|t| {
            let start = t.start.offset;
            let end = start + t.value.len() as u32;
            Location { uri: url.clone(), range: range_at_offsets(file, start, end) }
        })
        .collect()
}
