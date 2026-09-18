//! Token-at-position lookup, shared by hover / definition / document
//! highlight / completion.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

use foldhash::HashMap;
use mago_database::file::File as MagoFile;
use mago_database::file::FileId;
use mago_span::Position;
use mago_syntax::lexer::Lexer;
use mago_syntax::settings::LexerSettings;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;
use mago_syntax_core::input::Input;

/// Variable token (`$foo`) under the cursor.
///
/// Resolved name lookups go through [`mago_names::ResolvedNames`]
/// directly. Variables are not tracked there, so this byte-level scan
/// handles the only case the resolution map can't.
#[derive(Debug, Clone, Copy)]
pub struct VarAtCursor<'file> {
    /// Identifier text including the leading `$`.
    pub raw: &'file [u8],
    /// Identifier text without the leading `$`.
    pub name: &'file [u8],
    pub start: u32,
    pub end: u32,
}

/// Find the variable token (`$foo`) whose span covers `offset`. Operates
/// on file bytes directly: walks back from the cursor to the `$` and
/// forward to the end of the identifier. No lex required.
#[must_use]
pub fn variable_at_offset(file: &MagoFile, offset: u32) -> Option<VarAtCursor<'_>> {
    let bytes = file.contents.as_ref();
    let off = offset as usize;
    if off >= bytes.len() {
        return None;
    }

    let dollar = if bytes[off] == b'$' {
        off
    } else if is_var_char(bytes[off]) {
        let mut s = off;
        while s > 0 && is_var_char(bytes[s - 1]) {
            s -= 1;
        }
        if s == 0 || bytes[s - 1] != b'$' {
            return None;
        }
        s - 1
    } else {
        return None;
    };

    let name_start = dollar + 1;
    if name_start >= bytes.len() || !is_var_first_char(bytes[name_start]) {
        return None;
    }

    let mut end = name_start;
    while end < bytes.len() && is_var_char(bytes[end]) {
        end += 1;
    }

    let raw = &bytes[dollar..end];
    let name = &bytes[name_start..end];
    Some(VarAtCursor { raw, name, start: dollar as u32, end: end as u32 })
}

fn is_var_first_char(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphabetic()
}

fn is_var_char(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

/// Lex `file` into a token vector.
///
/// Backed by the per-file [`CacheEntry`] so repeated capability calls on the
/// same file skip the state-machine lex entirely; the only per-call cost is
/// the `Vec<Token<'_>>` reconstruction from cached offsets.
#[must_use]
pub fn lex(file: &MagoFile) -> Vec<Token<'_>> {
    let entry = cached_entry(file);
    let bytes = file.contents.as_ref();
    entry
        .tokens
        .iter()
        .map(|r| Token {
            kind: r.kind,
            start: Position { offset: r.start },
            value: &bytes[r.start as usize..r.end as usize],
        })
        .collect()
}

/// Drop cached lex entries for the given files.
///
/// Called when files change so the next [`lex`] call rebuilds. The hash-check
/// path also catches stale entries, but eager invalidation prevents the cache
/// from growing with versions of the same file.
pub fn invalidate(file_ids: &[FileId]) {
    if let Ok(mut guard) = cache().lock() {
        for id in file_ids {
            guard.remove(id);
        }
    }
}

/// Returns `true` if a token is whitespace or a comment.
#[must_use]
pub fn is_trivia(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Whitespace
            | TokenKind::SingleLineComment
            | TokenKind::HashComment
            | TokenKind::MultiLineComment
            | TokenKind::DocBlockComment
    )
}

#[derive(Clone, Copy, Debug)]
struct RawToken {
    kind: TokenKind,
    start: u32,
    end: u32,
}

#[derive(Debug)]
struct CacheEntry {
    tokens: Vec<RawToken>,
}

const LEX_CACHE_CAP: usize = 1024;

type LexCache = Mutex<HashMap<FileId, (u64, Arc<CacheEntry>)>>;

fn cache() -> &'static LexCache {
    static LEX_CACHE: OnceLock<LexCache> = OnceLock::new();
    LEX_CACHE.get_or_init(|| Mutex::new(HashMap::default()))
}

fn cached_entry(file: &MagoFile) -> Arc<CacheEntry> {
    let hash = xxhash_rust::xxh3::xxh3_64(&file.contents);
    if let Ok(guard) = cache().lock()
        && let Some((h, t)) = guard.get(&file.id)
        && *h == hash
    {
        return Arc::clone(t);
    }

    let entry = Arc::new(CacheEntry { tokens: lex_uncached(file) });
    if let Ok(mut guard) = cache().lock() {
        if guard.len() >= LEX_CACHE_CAP
            && let Some(k) = guard.keys().next().copied()
        {
            guard.remove(&k);
        }
        guard.insert(file.id, (hash, Arc::clone(&entry)));
    }
    entry
}

fn lex_uncached(file: &MagoFile) -> Vec<RawToken> {
    let input = Input::new(file.id, file.contents.as_ref());
    let mut lexer = Lexer::new(input, LexerSettings::default());
    let mut out = Vec::new();
    while let Some(result) = lexer.advance() {
        if let Ok(t) = result {
            let len = t.value.len() as u32;
            out.push(RawToken { kind: t.kind, start: t.start.offset, end: t.start.offset + len });
        }
    }
    out
}
