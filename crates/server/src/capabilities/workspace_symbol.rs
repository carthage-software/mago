//! `get_symbols`: fuzzy workspace-symbol search.
//!
//! Filters the codebase by a substring name query and returns matching
//! class-likes, free functions, and global constants declared in user files
//! (prelude builtins are skipped). Methods aren't enumerated yet.

use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::symbol::SymbolKind as MagoSymbolKind;
use mago_database::DatabaseReader;
use mago_database::file::FileType;
use mago_span::Span as FileSpan;
use mago_word::Word;

use crate::Server;
use crate::domain::Range;
use crate::domain::SymbolKind;
use crate::domain::SymbolLocation;
use crate::domain::WorkspaceSymbolItem;

const MAX_RESULTS: usize = 256;

impl Server {
    /// Up to [`MAX_RESULTS`] user-declared symbols whose name contains `query`
    /// (case-insensitive).
    #[must_use]
    pub fn get_symbols(&self, query: &str) -> Vec<WorkspaceSymbolItem> {
        let needle = query.as_bytes().to_ascii_lowercase();
        let codebase = self.codebase();
        let mut out = Vec::new();

        for meta in codebase.class_likes.values() {
            if !matches(&meta.name, &needle) || !self.is_user_symbol(meta.span) {
                continue;
            }

            out.push(WorkspaceSymbolItem {
                name: String::from_utf8_lossy(meta.original_name.as_bytes()).into_owned(),
                kind: classlike_kind(meta.kind),
                location: location_of(meta.span),
            });

            if out.len() >= MAX_RESULTS {
                return out;
            }
        }

        for ((_, name), meta) in codebase.function_likes.iter() {
            if matches!(meta.kind, FunctionLikeKind::Method)
                || !matches(name, &needle)
                || !self.is_user_symbol(meta.span)
            {
                continue;
            }

            out.push(WorkspaceSymbolItem {
                name: String::from_utf8_lossy(meta.original_name.as_bytes()).into_owned(),
                kind: SymbolKind::Function,
                location: location_of(meta.span),
            });

            if out.len() >= MAX_RESULTS {
                return out;
            }
        }

        for meta in codebase.constants.values() {
            if !matches(&meta.name, &needle) || !self.is_user_symbol(meta.span) {
                continue;
            }

            out.push(WorkspaceSymbolItem {
                name: String::from_utf8_lossy(meta.name.as_bytes()).into_owned(),
                kind: SymbolKind::Constant,
                location: location_of(meta.span),
            });

            if out.len() >= MAX_RESULTS {
                return out;
            }
        }

        out
    }

    /// Whether `span`'s file is a user file (not a prelude builtin). Builtins
    /// dominate results with `int`, `array_map`, etc., which editors already
    /// surface via stubs.
    fn is_user_symbol(&self, span: FileSpan) -> bool {
        self.database().get(&span.file_id).map(|f| !matches!(f.file_type, FileType::Builtin)).unwrap_or(false)
    }
}

fn location_of(span: FileSpan) -> SymbolLocation {
    SymbolLocation { file: span.file_id, range: Range::new(span.start.offset, span.end.offset) }
}

/// Substring match against the lowercased FQCN/FQN.
fn matches(haystack: &Word, needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }

    memchr::memmem::find(haystack.as_bytes(), needle).is_some()
}

fn classlike_kind(kind: MagoSymbolKind) -> SymbolKind {
    match kind {
        MagoSymbolKind::Class => SymbolKind::Class,
        MagoSymbolKind::Interface => SymbolKind::Interface,
        MagoSymbolKind::Trait => SymbolKind::Trait,
        MagoSymbolKind::Enum => SymbolKind::Enum,
    }
}
