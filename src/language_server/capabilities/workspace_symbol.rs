//! `workspace/symbol`.
//!
//! Filters [`mago_codex::metadata::CodebaseMetadata`] by a fuzzy name query
//! and returns matching classes, interfaces, traits, enums, functions, and
//! global constants; across every file mago has scanned, including
//! prelude builtins. Methods are not yet enumerated (would require a
//! per-class flatten); we'll add them when class navigation needs them.

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::symbol::SymbolKind as MagoSymbolKind;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_span::Span;
use mago_word::Word;
use tower_lsp::lsp_types::Location;
use tower_lsp::lsp_types::SymbolInformation;
use tower_lsp::lsp_types::SymbolKind as LspSymbolKind;
use tower_lsp::lsp_types::Url;

use crate::language_server::position::range_at_offsets;

const MAX_RESULTS: usize = 256;

pub fn compute(database: &Database<'_>, codebase: &CodebaseMetadata, query: &str) -> Vec<SymbolInformation> {
    let needle = query.as_bytes().to_ascii_lowercase();
    let mut out = Vec::new();

    for meta in codebase.class_likes.values() {
        if !matches(&meta.name, &needle) {
            continue;
        }
        if !is_user_symbol(database, meta.span) {
            continue;
        }
        if let Some(location) = span_to_location(database, meta.span) {
            #[allow(deprecated)]
            out.push(SymbolInformation {
                name: String::from_utf8_lossy(meta.original_name.as_bytes()).into_owned(),
                kind: classlike_kind(meta.kind),
                tags: None,
                deprecated: None,
                location,
                container_name: None,
            });
        }
        if out.len() >= MAX_RESULTS {
            return out;
        }
    }

    for ((_, name), meta) in codebase.function_likes.iter() {
        if matches!(meta.kind, mago_codex::metadata::function_like::FunctionLikeKind::Method) {
            continue;
        }
        if !matches(name, &needle) {
            continue;
        }
        if !is_user_symbol(database, meta.span) {
            continue;
        }
        if let Some(location) = span_to_location(database, meta.span) {
            #[allow(deprecated)]
            out.push(SymbolInformation {
                name: meta
                    .original_name
                    .map(|n| String::from_utf8_lossy(n.as_bytes()).into_owned())
                    .unwrap_or_else(|| String::from_utf8_lossy(name.as_bytes()).into_owned()),
                kind: LspSymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location,
                container_name: None,
            });
        }
        if out.len() >= MAX_RESULTS {
            return out;
        }
    }

    for meta in codebase.constants.values() {
        if !matches(&meta.name, &needle) {
            continue;
        }
        if !is_user_symbol(database, meta.span) {
            continue;
        }
        if let Some(location) = span_to_location(database, meta.span) {
            #[allow(deprecated)]
            out.push(SymbolInformation {
                name: String::from_utf8_lossy(meta.name.as_bytes()).into_owned(),
                kind: LspSymbolKind::CONSTANT,
                tags: None,
                deprecated: None,
                location,
                container_name: None,
            });
        }
        if out.len() >= MAX_RESULTS {
            return out;
        }
    }

    out
}

/// Substring match against the lowercased FQCN/FQN.
fn matches(haystack: &Word, needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    memchr::memmem::find(haystack.as_bytes(), needle).is_some()
}

/// Skip prelude builtins; the user almost always wants their own code, and
/// otherwise the result list is dominated by `int`, `string`, `array_map`
/// etc. Editors expose those via stubs already.
fn is_user_symbol(database: &Database<'_>, span: Span) -> bool {
    database.get(&span.file_id).map(|f| !matches!(f.file_type, FileType::Builtin)).unwrap_or(false)
}

fn classlike_kind(k: MagoSymbolKind) -> LspSymbolKind {
    match k {
        MagoSymbolKind::Class => LspSymbolKind::CLASS,
        MagoSymbolKind::Interface => LspSymbolKind::INTERFACE,
        MagoSymbolKind::Trait => LspSymbolKind::CLASS,
        MagoSymbolKind::Enum => LspSymbolKind::ENUM,
    }
}

fn span_to_location(database: &Database<'_>, span: Span) -> Option<Location> {
    let file = database.get(&span.file_id).ok()?;
    let path = file.path.as_ref()?;
    let url = Url::from_file_path(path).ok()?;
    let range = range_at_offsets(&file, span.start.offset, span.end.offset);
    Some(Location { uri: url, range })
}
