//! `textDocument/definition`.
//!
//! Resolves the identifier under the cursor to its fully-qualified name, looks
//! up that symbol's declaration span in the codebase, and converts it to an LSP
//! [`Location`].

use mago_codex::metadata::CodebaseMetadata;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_names::ResolvedNames;
use mago_span::Span;
use tower_lsp_server::ls_types::Location;
use tower_lsp_server::ls_types::Uri;

use crate::language_server::position::range_at_offsets;

pub fn compute(
    database: &Database<'_>,
    codebase: &CodebaseMetadata,
    resolved: &ResolvedNames<'_>,
    offset: u32,
) -> Option<Location> {
    let (_, _, fqcn, _) = resolved.at_offset(offset)?;
    let span = codebase.span_of(fqcn)?;
    span_to_location(database, span)
}

fn span_to_location(database: &Database<'_>, span: Span) -> Option<Location> {
    let file = database.get(&span.file_id).ok()?;
    let path = file.path.as_ref()?;
    let url = Uri::from_file_path(path)?;
    let range = range_at_offsets(&file, span.start.offset, span.end.offset);
    Some(Location { uri: url, range })
}
