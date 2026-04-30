//! `textDocument/references`.
//!
//! Resolves the symbol under the cursor to its fully-qualified name via
//! [`mago_names::ResolvedNames`], then iterates every host file in the
//! workspace, consulting *that file's* `ResolvedNames` for entries that
//! resolve to the same FQN. This handles aliased imports correctly:
//!
//! ```php
//! namespace Foo;
//! use Bar as Qux;
//! $x = Qux\G;        // resolves to Bar\G
//! ```
//!
//! Token-based scanning would miss this; the source spelling is `Qux\G`
//! but the symbol is `Bar\G`. The resolver records the resolved FQN, so
//! comparing FQNs across files is the only correct strategy.
//!
//! A coarse byte-level case-insensitive substring filter on the raw
//! file contents skips files that can't possibly contain the local
//! name, avoiding parse + resolve on irrelevant files.
//!
//! Variables (`$foo`) aren't tracked by `ResolvedNames`; they fall
//! through to a same-file token scan.
//!
//! Only `FileType::Host` is searched; prelude / vendored sources
//! aren't user-facing navigation targets.

use std::sync::Arc;

use mago_codex::metadata::CodebaseMetadata;
use mago_database::DatabaseReader;
use mago_database::file::File as MagoFile;
use mago_database::file::FileType;
use mago_span::Span;
use mago_syntax::token::TokenKind;
use tower_lsp::lsp_types::Location;
use tower_lsp::lsp_types::Url;

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

    let local_lower = target_fqn.rsplit('\\').next().unwrap_or(target_fqn).to_ascii_lowercase();

    let declaration_span =
        if include_declaration { None } else { declaration_name_span(workspace.service.codebase(), target_fqn) };

    let candidates: Vec<Arc<MagoFile>> = workspace
        .database
        .files()
        .filter(|f| matches!(f.file_type, FileType::Host))
        .filter(|f| contains_ascii_ci(f.contents.as_bytes(), local_lower.as_bytes()))
        .collect();

    let mut out = Vec::new();
    for arc_file in candidates {
        let Some(analysis) = workspace.file_analysis_for(arc_file.id) else { continue };
        let Some(path) = arc_file.path.as_ref() else { continue };
        let Ok(url) = Url::from_file_path(path) else { continue };

        for (start, end, name, _) in analysis.resolved().iter() {
            if !name.eq_ignore_ascii_case(target_fqn) {
                continue;
            }
            if let Some(decl) = declaration_span
                && decl.file_id == arc_file.id
                && decl.start.offset == start
            {
                continue;
            }
            out.push(Location { uri: url.clone(), range: range_at_offsets(&arc_file, start, end) });
        }
    }

    out
}

fn declaration_name_span(codebase: &CodebaseMetadata, fqn: &str) -> Option<Span> {
    if let Some(meta) = codebase.get_class_like(fqn) {
        return meta.name_span;
    }
    if let Some(meta) = codebase.get_function(fqn) {
        return meta.name_span;
    }
    if let Some(meta) = codebase.get_constant(fqn) {
        return Some(meta.span);
    }
    None
}

fn contains_ascii_ci(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }
    let last = haystack.len() - needle.len();
    for i in 0..=last {
        if haystack[i..i + needle.len()].iter().zip(needle).all(|(a, b)| a.eq_ignore_ascii_case(b)) {
            return true;
        }
    }
    false
}

fn same_file_variable_locations(file: &MagoFile, raw: &str) -> Vec<Location> {
    let Some(path) = file.path.as_ref() else {
        return Vec::new();
    };
    let Ok(url) = Url::from_file_path(path) else {
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
