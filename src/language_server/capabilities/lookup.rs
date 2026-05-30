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
pub struct VarAtCursor<'a> {
    /// Identifier text including the leading `$`.
    pub raw: &'a [u8],
    /// Identifier text without the leading `$`.
    pub name: &'a [u8],
    pub start: u32,
    pub end: u32,
}

/// Find the variable token (`$foo`) whose span covers `offset`. Operates
/// on file bytes directly: walks back from the cursor to the `$` and
/// forward to the end of the identifier. No lex required.
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

/// Find the namespace in effect at `offset`. Returns `None` if no
/// `namespace` declaration covers the offset.
///
/// Backed by the same content-hash-keyed cache as [`lex`]: namespace
/// ranges are extracted once during the initial token scan and reused
/// across every capability call until the file changes.
pub fn namespace_at_offset(file: &MagoFile, offset: u32) -> Option<Vec<u8>> {
    let entry = cached_entry(file);
    entry.namespaces.iter().find(|r| r.start <= offset && offset < r.end).map(|r| r.name.clone().into_vec())
}

/// What a `use` statement imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportKind {
    Class,
    Function,
    Constant,
}

/// One name brought into scope by a `use` statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub fqcn_lower: Vec<u8>,
    pub alias: Vec<u8>,
    pub kind: ImportKind,
}

/// Collect every `use` import declared before `offset`. Handles aliases
/// (`as`), grouped imports (`use Foo\{Bar, Baz as Q}`), and the `function` /
/// `const` variants. Backed by the same cached token scan as [`lex`].
pub fn imports_at_offset(file: &MagoFile, offset: u32) -> Vec<Import> {
    let entry = cached_entry(file);
    let bytes = file.contents.as_ref();
    let toks = &entry.tokens;
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        if toks[i].start >= offset {
            break;
        }
        if matches!(toks[i].kind, TokenKind::Use) {
            i = parse_use(toks, bytes, i + 1, &mut out);
        } else {
            i += 1;
        }
    }
    out
}

fn parse_use(toks: &[RawToken], bytes: &[u8], mut j: usize, out: &mut Vec<Import>) -> usize {
    j = skip_trivia(toks, j);
    let mut kind = ImportKind::Class;
    match toks.get(j).map(|t| t.kind) {
        Some(TokenKind::Function) => {
            kind = ImportKind::Function;
            j = skip_trivia(toks, j + 1);
        }
        Some(TokenKind::Const) => {
            kind = ImportKind::Constant;
            j = skip_trivia(toks, j + 1);
        }
        _ => {}
    }

    loop {
        let (path, nj) = read_name(toks, bytes, j);
        j = skip_trivia(toks, nj);

        match toks.get(j).map(|t| t.kind) {
            Some(TokenKind::LeftBrace) => {
                let prefix = trim_path(&path);
                j = skip_trivia(toks, j + 1);
                loop {
                    let (sub, sj) = read_name(toks, bytes, j);
                    j = skip_trivia(toks, sj);
                    let trimmed = trim_path(&sub);
                    let mut alias = last_segment(&trimmed).to_vec();
                    if matches!(toks.get(j).map(|t| t.kind), Some(TokenKind::As)) {
                        j = skip_trivia(toks, j + 1);
                        let (al, aj) = read_name(toks, bytes, j);
                        alias = trim_path(&al);
                        j = skip_trivia(toks, aj);
                    }
                    let mut fqcn = prefix.clone();
                    fqcn.push(b'\\');
                    fqcn.extend_from_slice(&trimmed);
                    push_import(out, fqcn, alias, kind);

                    match toks.get(j).map(|t| t.kind) {
                        Some(TokenKind::Comma) => {
                            j = skip_trivia(toks, j + 1);
                            if matches!(toks.get(j).map(|t| t.kind), Some(TokenKind::RightBrace)) {
                                j += 1;
                                break;
                            }
                        }
                        Some(TokenKind::RightBrace) => {
                            j += 1;
                            break;
                        }
                        _ => break,
                    }
                }
                j = skip_trivia(toks, j);
                return match toks.get(j).map(|t| t.kind) {
                    Some(TokenKind::Semicolon) => j + 1,
                    _ => j,
                };
            }
            Some(TokenKind::As) => {
                j = skip_trivia(toks, j + 1);
                let (al, aj) = read_name(toks, bytes, j);
                push_import(out, trim_path(&path), trim_path(&al), kind);
                j = skip_trivia(toks, aj);
            }
            _ => {
                let p = trim_path(&path);
                let alias = last_segment(&p).to_vec();
                push_import(out, p, alias, kind);
            }
        }

        match toks.get(j).map(|t| t.kind) {
            Some(TokenKind::Comma) => j = skip_trivia(toks, j + 1),
            Some(TokenKind::Semicolon) => return j + 1,
            _ => return j,
        }
    }
}

fn push_import(out: &mut Vec<Import>, fqcn: Vec<u8>, alias: Vec<u8>, kind: ImportKind) {
    if fqcn.is_empty() || alias.is_empty() {
        return;
    }
    out.push(Import { fqcn_lower: fqcn.to_ascii_lowercase(), alias, kind });
}

fn read_name(toks: &[RawToken], bytes: &[u8], mut j: usize) -> (Vec<u8>, usize) {
    let mut name = Vec::new();
    while let Some(t) = toks.get(j) {
        match t.kind {
            TokenKind::Identifier
            | TokenKind::QualifiedIdentifier
            | TokenKind::FullyQualifiedIdentifier
            | TokenKind::NamespaceSeparator => {
                name.extend_from_slice(&bytes[t.start as usize..t.end as usize]);
                j += 1;
            }
            _ => break,
        }
    }
    (name, j)
}

fn skip_trivia(toks: &[RawToken], mut j: usize) -> usize {
    while toks.get(j).is_some_and(|t| is_trivia(t.kind)) {
        j += 1;
    }
    j
}

fn trim_path(path: &[u8]) -> Vec<u8> {
    let trimmed = path.strip_prefix(b"\\").unwrap_or(path);
    trimmed.strip_suffix(b"\\").unwrap_or(trimmed).to_vec()
}

fn last_segment(path: &[u8]) -> &[u8] {
    match memchr::memrchr(b'\\', path) {
        Some(i) => &path[i + 1..],
        None => path,
    }
}

/// Lex `file` into a token vector. Backed by the per-file [`CacheEntry`]
/// so repeated capability calls on the same file skip the state-machine
/// lex entirely; the only per-call cost is the `Vec<Token<'_>>`
/// reconstruction from cached offsets.
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

/// Drop cached lex entries for the given files. Called from the backend
/// when files change so the next [`lex`] call rebuilds. The hash-check
/// path also catches stale entries, but eager invalidation prevents the
/// cache from growing with versions of the same file.
pub fn invalidate(file_ids: &[FileId]) {
    if let Ok(mut guard) = cache().lock() {
        for id in file_ids {
            guard.remove(id);
        }
    }
}

/// Returns `true` if a token is whitespace or a comment.
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
struct NamespaceRange {
    start: u32,
    end: u32,
    name: Box<[u8]>,
}

#[derive(Debug)]
struct CacheEntry {
    tokens: Vec<RawToken>,
    namespaces: Vec<NamespaceRange>,
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

    let entry = Arc::new(build_entry(file));
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

fn build_entry(file: &MagoFile) -> CacheEntry {
    let tokens = lex_uncached(file);
    let namespaces = collect_namespaces(file, &tokens);
    CacheEntry { tokens, namespaces }
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

fn collect_namespaces(file: &MagoFile, tokens: &[RawToken]) -> Vec<NamespaceRange> {
    let bytes = file.contents.as_ref();
    let file_size = file.size;
    let mut out: Vec<NamespaceRange> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if !matches!(tokens[i].kind, TokenKind::Namespace) {
            i += 1;
            continue;
        }
        let header_start = tokens[i].start;

        let mut j = i + 1;
        while j < tokens.len() && is_trivia(tokens[j].kind) {
            j += 1;
        }

        let mut name: Vec<u8> = Vec::new();
        while j < tokens.len()
            && matches!(
                tokens[j].kind,
                TokenKind::Identifier | TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier
            )
        {
            let s = tokens[j].start as usize;
            let e = tokens[j].end as usize;
            name.extend_from_slice(mago_bytes::trim_start_byte(&bytes[s..e], b'\\'));
            j += 1;
        }

        while j < tokens.len() && is_trivia(tokens[j].kind) {
            j += 1;
        }

        let (range_end, advance) = match tokens.get(j).map(|t| t.kind) {
            Some(TokenKind::LeftBrace) => match find_matching_brace(tokens, j) {
                Some(close_end) => (close_end, position_after(tokens, close_end)),
                None => (file_size, tokens.len()),
            },
            _ => (file_size, j + 1),
        };

        out.push(NamespaceRange { start: header_start, end: range_end, name: Box::from(name) });
        i = advance;
    }

    out.sort_by_key(|r| r.start);
    out
}

fn find_matching_brace(tokens: &[RawToken], open_idx: usize) -> Option<u32> {
    let mut depth: i32 = 0;
    for t in tokens.iter().skip(open_idx) {
        match t.kind {
            TokenKind::LeftBrace => depth += 1,
            TokenKind::RightBrace => {
                depth -= 1;
                if depth == 0 {
                    return Some(t.end);
                }
            }
            _ => {}
        }
    }
    None
}

fn position_after(tokens: &[RawToken], offset: u32) -> usize {
    tokens.iter().position(|t| t.start >= offset).unwrap_or(tokens.len())
}
