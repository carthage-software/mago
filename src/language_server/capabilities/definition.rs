//! `textDocument/definition`.
//!
//! Resolves the identifier under the cursor via [`mago_names::ResolvedNames`]
//! and converts the resulting metadata span into an LSP [`Location`].

use mago_codex::metadata::CodebaseMetadata;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_names::ResolvedNames;
use mago_span::Span;
use tower_lsp::lsp_types::Location;
use tower_lsp::lsp_types::Url;

use crate::language_server::position::range_at_offsets;

pub fn compute(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    resolved: &ResolvedNames<'_>,
    offset: u32,
) -> Option<Location> {
    let (_, _, fqcn, _) = resolved.at_offset(offset)?;
    let span = resolve_span(codebase, fqcn)?;
    span_to_location(database, span)
}

fn resolve_span(codebase: &CodebaseMetadata, fqcn: &str) -> Option<Span> {
    if let Some(meta) = codebase.get_class_like(fqcn) {
        return Some(meta.name_span.unwrap_or(meta.span));
    }
    if let Some(meta) = codebase.get_function(fqcn) {
        return Some(meta.name_span.unwrap_or(meta.span));
    }
    if let Some(meta) = codebase.get_constant(fqcn) {
        return Some(meta.span);
    }
    None
}

fn span_to_location(database: &Database<'_>, span: Span) -> Option<Location> {
    let file = database.get(&span.file_id).ok()?;
    let path = file.path.as_ref()?;
    let url = Url::from_file_path(path).ok()?;
    let range = range_at_offsets(&file, span.start.offset, span.end.offset);
    Some(Location { uri: url, range })
}
