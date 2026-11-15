//! Cached tokens shared by editor queries.

mod docblocks;

use std::sync::Arc;
use std::sync::Mutex;

use foldhash::HashMap;
use mago_database::file::File as MagoFile;
use mago_database::file::FileId;
use mago_span::Position;
use mago_syntax::lexer::Lexer;
use mago_syntax::settings::LexerSettings;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;
use mago_syntax_core::input::Input;

pub(crate) use docblocks::DocblockAnchor;
pub(crate) use docblocks::DocblockSource;

fn is_symbol_name(kind: TokenKind) -> bool {
    kind.is_identifier_maybe_reserved()
        || matches!(kind, TokenKind::QualifiedIdentifier | TokenKind::FullyQualifiedIdentifier | TokenKind::Variable)
}

/// A workspace's token cache, capped at 32 MiB and 1,024 files.
/// Lexer settings stay fixed for its lifetime.
pub(crate) struct TokenCache {
    settings: LexerSettings,
    entries: Mutex<LexCache>,
}

impl TokenCache {
    pub(crate) fn new(settings: LexerSettings) -> Self {
        Self { settings, entries: Mutex::default() }
    }

    pub(crate) fn invalidate(&self, file_ids: impl IntoIterator<Item = FileId>) {
        if let Ok(mut cache) = self.entries.lock() {
            for id in file_ids {
                cache.remove(id);
            }
        }
    }

    /// Lex `file` into a token vector.
    ///
    /// Backed by the per-file [`CacheEntry`] so repeated capability calls on the
    /// same file skip the state-machine lex entirely; the only per-call cost is
    /// the `Vec<Token<'_>>` reconstruction from cached offsets.
    #[must_use]
    pub(crate) fn lex<'file>(&self, file: &'file MagoFile) -> Vec<Token<'file>> {
        let entry = self.entry(file);
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

    /// Iterate cached tokens without rebuilding the file's token vector.
    pub(crate) fn tokens<'file>(&self, file: &'file MagoFile) -> impl Iterator<Item = Token<'file>> {
        let entry = self.entry(file);
        let bytes = file.contents.as_ref();
        (0..entry.tokens.len()).map(move |index| {
            let token = entry.tokens[index];
            Token {
                kind: token.kind,
                start: Position { offset: token.start },
                value: &bytes[token.start as usize..token.end as usize],
            }
        })
    }

    /// Borrow one cached token without rebuilding the file's token vector.
    pub(crate) fn at_offset<'file>(&self, file: &'file MagoFile, offset: u32) -> Option<Token<'file>> {
        let entry = self.entry(file);
        let index = entry.tokens.partition_point(|token| token.end <= offset);
        let token = entry.tokens.get(index).filter(|token| token.start <= offset)?;
        Some(Token {
            kind: token.kind,
            start: Position::new(token.start),
            value: &file.contents[token.start as usize..token.end as usize],
        })
    }

    /// Let a symbol query select a name whose right edge touches the caret.
    /// Keep exact offsets for ranges, completion, and signature help.
    #[must_use]
    fn symbol_offset(&self, file: &MagoFile, offset: u32) -> u32 {
        let entry = self.entry(file);
        let index = entry.tokens.partition_point(|token| token.end <= offset);
        if let Some(token) = entry.tokens.get(index)
            && token.start <= offset
            && is_symbol_name(token.kind)
        {
            // A namespace alias ends inside the qualified-name token.
            return if offset > token.start && file.contents.get(offset as usize) == Some(&b'\\') {
                offset - 1
            } else {
                offset
            };
        }
        match index.checked_sub(1).and_then(|index| entry.tokens.get(index)) {
            Some(token) if token.end == offset && is_symbol_name(token.kind) => offset.saturating_sub(1),
            _ => offset,
        }
    }

    pub(crate) fn docblocks(&self, file: Arc<MagoFile>) -> DocblockSource {
        DocblockSource { entry: self.entry(&file), file }
    }

    fn entry(&self, file: &MagoFile) -> Arc<CacheEntry> {
        let hash = xxhash_rust::xxh3::xxh3_64(&file.contents);
        if let Ok(cache) = self.entries.lock()
            && let Some((cached_hash, entry)) = cache.entries.get(&file.id)
            && *cached_hash == hash
        {
            return Arc::clone(entry);
        }

        let entry = Arc::new(CacheEntry { tokens: lex_uncached(file, self.settings) });
        if let Ok(mut cache) = self.entries.lock() {
            cache.insert(file.id, hash, &entry, LEX_CACHE_BYTES);
        }
        entry
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
const LEX_CACHE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Default)]
struct LexCache {
    entries: HashMap<FileId, (u64, Arc<CacheEntry>)>,
    bytes: usize,
}

impl LexCache {
    fn remove(&mut self, id: FileId) {
        if let Some((_, entry)) = self.entries.remove(&id) {
            self.bytes -= entry.tokens.capacity() * size_of::<RawToken>();
        }
    }

    fn insert(&mut self, id: FileId, hash: u64, entry: &Arc<CacheEntry>, byte_limit: usize) {
        self.remove(id);
        let bytes = entry.tokens.capacity() * size_of::<RawToken>();
        if bytes > byte_limit {
            return;
        }
        while self.entries.len() >= LEX_CACHE_CAP || self.bytes + bytes > byte_limit {
            let Some(id) = self.entries.keys().next().copied() else { break };
            self.remove(id);
        }
        self.bytes += bytes;
        self.entries.insert(id, (hash, Arc::clone(entry)));
    }
}

impl crate::Server {
    pub(crate) fn lex<'file>(&self, file: &'file MagoFile) -> Vec<Token<'file>> {
        self.tokens.lex(file)
    }

    /// Let a symbol query select a name whose right edge touches the caret.
    #[must_use]
    pub fn symbol_offset(&self, file: &MagoFile, offset: u32) -> u32 {
        self.tokens.symbol_offset(file, offset)
    }

    pub(crate) fn docblock_before<'file>(
        &self,
        file: &'file MagoFile,
        offset: u32,
        anchor: DocblockAnchor,
    ) -> Option<&'file [u8]> {
        self.tokens.entry(file).docblock_before(file, offset, anchor)
    }
}

fn lex_uncached(file: &MagoFile, settings: LexerSettings) -> Vec<RawToken> {
    let input = Input::new(file.id, file.contents.as_ref());
    let mut lexer = Lexer::new(input, settings);
    let mut out = Vec::new();
    while let Some(result) = lexer.advance() {
        if let Ok(t) = result {
            let len = t.value.len() as u32;
            out.push(RawToken { kind: t.kind, start: t.start.offset, end: t.start.offset + len });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_offsets_select_only_names_at_the_caret() {
        for (source, selects_left) in [
            ("<?php Éclair|();", true),
            (r"<?php \App\Factory|::make();", true),
            (r"<?php Alias|\Thing();", true),
            (r"<?php Alias\|Thing();", false),
            ("<?php $object->match|();", true),
            ("<?php $value|", true),
            ("<?php $value|->method();", true),
            ("<?php $value|$other", false),
            ("<?php $va|lue", false),
            ("<?php make |();", false),
            ("<?php // make|\n", false),
            ("<?php 'make|'", false),
        ] {
            let Some(offset) = source.find('|') else { panic!("cursor marker missing") };
            let offset = offset as u32;
            let file =
                MagoFile::ephemeral(b"cursor.php".as_slice().into(), source.replace('|', "").into_bytes().into());
            assert_eq!(
                TokenCache::new(LexerSettings::default()).symbol_offset(&file, offset),
                offset - u32::from(selects_left),
                "{source}"
            );
        }
    }

    #[test]
    fn attached_docblocks_do_not_cross_code_or_read_strings() {
        let source = b"<?php /** Class docs. */ class A { /** Property docs. */ #[Mark('/** fake */', [1, 2])] public int $first, $second; /** Old docs. */ public int $old = 1; public int $bare; }";
        let file = MagoFile::ephemeral(b"docs.php".as_slice().into(), source.as_slice().into());
        let docs = TokenCache::new(LexerSettings::default()).docblocks(Arc::new(file));
        for (name, anchor, expected) in [
            (b"class A".as_slice(), DocblockAnchor::DeclarationStart, Some(b"/** Class docs. */".as_slice())),
            (b"$first".as_slice(), DocblockAnchor::DeclarationName, Some(b"/** Property docs. */".as_slice())),
            (b"$second".as_slice(), DocblockAnchor::DeclarationName, Some(b"/** Property docs. */".as_slice())),
            (b"$bare".as_slice(), DocblockAnchor::DeclarationName, None),
        ] {
            let Some(offset) = memchr::memmem::find(source, name) else { panic!("fixture token missing") };
            assert_eq!(docs.before(offset as u32, anchor), expected);
        }
    }

    #[test]
    fn docblock_sources_keep_matching_tokens_without_retaining_old_files() {
        let source = b"<?php /** Original. */ class Example {}";
        let Some(offset) = memchr::memmem::find(source, b"class") else { panic!("fixture token missing") };
        let file = Arc::new(MagoFile::ephemeral(b"docs.php".as_slice().into(), source.as_slice().into()));
        let weak_file = Arc::downgrade(&file);
        let cache = TokenCache::new(LexerSettings::default());
        let docs = cache.docblocks(Arc::clone(&file));
        let weak_tokens = Arc::downgrade(&docs.entry);
        let replacement = b"<?php /** A longer replacement. */ class Example {}";
        let Some(changed_offset) = memchr::memmem::find(replacement, b"class") else { panic!("fixture token missing") };
        let changed = cache.docblocks(Arc::new(MagoFile::ephemeral(file.name.clone(), replacement.as_slice().into())));

        assert_eq!(docs.before(offset as u32, DocblockAnchor::DeclarationStart), Some(b"/** Original. */".as_slice()));
        assert_eq!(
            changed.before(changed_offset as u32, DocblockAnchor::DeclarationStart),
            Some(b"/** A longer replacement. */".as_slice())
        );
        cache.invalidate([file.id]);
        drop(file);
        drop(docs);
        assert!(weak_file.upgrade().is_none());
        assert!(weak_tokens.upgrade().is_none());
    }

    #[test]
    fn grouped_declarations_share_docblocks_without_crossing_boundaries() {
        for (source, expected) in [
            ("<?php /** Shared. */ const A = 1, B = [2, 3], |C = 4;", Some("/** Shared. */")),
            (
                "<?php class A { /** Shared. */ #[Mark([1, 2])] final public const array A = ['docs' => '/** Fake. */', 'items' => [1, 2]], |B = []; }",
                Some("/** Shared. */"),
            ),
            (
                "<?php class A { /** Shared. */ public array $first = [1, 2], $second = [(3 + 4)], |$third = []; }",
                Some("/** Shared. */"),
            ),
            ("<?php class A { /** Shared. */ public ?int $first, |$second; }", Some("/** Shared. */")),
            ("<?php /** Shared. */ const A = 1, /** Local. */ |B = 2;", Some("/** Local. */")),
            ("<?php /** Old. */ const A = 1; const B = 2, |C = 3;", None),
            ("<?php class A { /** Old. */ public int $first = 1; public int $second = 2, |$third = 3; }", None),
            ("<?php /** Other. */ function f(int $first, int |$second) {}", None),
            ("<?php /** Other. */ call(1, |SECOND);", None),
            ("<?php /** Other. */ const A = [1, |B = 2;", None),
        ] {
            let Some(offset) = source.find('|') else { panic!("cursor marker missing") };
            let file = MagoFile::ephemeral(b"docs.php".as_slice().into(), source.replace('|', "").into_bytes().into());
            assert_eq!(
                TokenCache::new(LexerSettings::default())
                    .docblocks(Arc::new(file))
                    .before(offset as u32, DocblockAnchor::DeclarationName),
                expected.map(str::as_bytes),
                "{source}",
            );
        }
    }

    #[test]
    fn lex_cache_respects_settings_and_file_contents() {
        let file = MagoFile::ephemeral(b"lexer-settings.php".as_slice().into(), b"<? echo 1;".as_slice().into());
        let enabled = TokenCache::new(LexerSettings { enable_short_tags: true });
        let disabled = TokenCache::new(LexerSettings { enable_short_tags: false });
        assert!(enabled.lex(&file).iter().any(|token| token.kind == TokenKind::Echo));
        assert_eq!(disabled.lex(&file).len(), 1);
        let cached = enabled.entry(&file);
        assert!(Arc::ptr_eq(&cached, &enabled.entry(&file)));
        let changed = MagoFile::ephemeral(file.name.clone(), b"<? return 2;".as_slice().into());
        assert!(enabled.lex(&changed).iter().any(|token| token.kind == TokenKind::Return));
        assert!(!Arc::ptr_eq(&cached, &enabled.entry(&changed)));
        enabled.invalidate([file.id]);
        let Ok(cache) = enabled.entries.lock() else { panic!("token cache lock poisoned") };
        assert!(!cache.entries.contains_key(&file.id));
        assert_eq!(disabled.lex(&file).len(), 1);
    }

    #[test]
    fn token_cache_counts_capacity_and_releases_evicted_entries() {
        let file = MagoFile::ephemeral(b"cache-budget.php".as_slice().into(), b"<?php echo 1;".as_slice().into());
        let entry = Arc::new(CacheEntry { tokens: lex_uncached(&file, LexerSettings::default()) });
        let limit = entry.tokens.capacity() * size_of::<RawToken>();
        let weak = Arc::downgrade(&entry);
        let mut cache = LexCache::default();
        cache.insert(file.id, 1, &entry, limit);
        cache.insert(file.id, 2, &entry, limit);
        assert_eq!(cache.bytes, limit);
        let second = Arc::new(CacheEntry { tokens: lex_uncached(&file, LexerSettings::default()) });
        let other = FileId::new(b"cache-other.php");
        cache.insert(other, 1, &second, limit);
        drop(entry);
        assert!(weak.upgrade().is_none());
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.bytes, limit);
        cache.insert(other, 2, &second, limit - 1);
        assert!(cache.entries.is_empty());
        assert_eq!(cache.bytes, 0);
    }

    #[test]
    fn dropping_one_workspace_releases_only_its_tokens() {
        let file = MagoFile::ephemeral(b"shared.php".as_slice().into(), b"<?php echo 1;".as_slice().into());
        let first = TokenCache::new(LexerSettings::default());
        let second = TokenCache::new(LexerSettings::default());
        let first_entry = first.entry(&file);
        let second_entry = second.entry(&file);
        assert!(!Arc::ptr_eq(&first_entry, &second_entry));
        let released = Arc::downgrade(&first_entry);
        drop(first_entry);
        drop(first);
        assert!(released.upgrade().is_none());
        assert!(Arc::ptr_eq(&second_entry, &second.entry(&file)));
    }
}
