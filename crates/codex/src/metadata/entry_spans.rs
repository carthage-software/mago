use mago_database::file::FileId;
use mago_span::Span;
use mago_word::Word;

use super::CodebaseEntryKeys;
use super::CodebaseMetadata;

/// Declaration spans needed to resolve file ownership after merging scans.
#[derive(Debug)]
pub struct CodebaseEntrySpans {
    class_likes: Vec<(Word, Span)>,
    class_like_aliases: Vec<(Word, Word, Span)>,
    function_likes: Vec<((Word, Word), Span)>,
    constants: Vec<(Word, Span)>,
    file_ids: Vec<FileId>,
}

impl CodebaseMetadata {
    /// Saves ownership candidates so the scan metadata can move into the merged codebase.
    #[must_use]
    pub fn entry_spans(&self) -> CodebaseEntrySpans {
        CodebaseEntrySpans {
            class_likes: self.class_likes.iter().map(|(name, metadata)| (*name, metadata.span)).collect(),
            class_like_aliases: self.class_like_alias_declarations().collect(),
            function_likes: self.function_likes.iter().map(|(key, metadata)| (*key, metadata.span)).collect(),
            constants: self.constants.iter().map(|(name, metadata)| (*name, metadata.span)).collect(),
            file_ids: self.file_signatures.keys().copied().collect(),
        }
    }
}

impl CodebaseEntrySpans {
    /// Keeps declarations whose full spans still match the merged entries.
    #[must_use]
    pub fn into_owned_keys(self, merged: &CodebaseMetadata) -> CodebaseEntryKeys {
        CodebaseEntryKeys {
            class_like_names: self
                .class_likes
                .iter()
                .filter(|(name, span)| merged.class_likes.get(name).is_some_and(|metadata| metadata.span == *span))
                .map(|(name, _)| *name)
                .collect(),
            class_like_aliases: self
                .class_like_aliases
                .into_iter()
                .filter(|(name, _, span)| {
                    merged
                        .class_like_alias_declarations
                        .get(name)
                        .is_some_and(|(_, merged_span, _)| merged_span == span)
                })
                .collect(),
            function_like_keys: self
                .function_likes
                .iter()
                .filter(|(key, span)| merged.function_likes.get(key).is_some_and(|metadata| metadata.span == *span))
                .map(|(key, _)| *key)
                .collect(),
            constant_names: self
                .constants
                .iter()
                .filter(|(name, span)| merged.constants.get(name).is_some_and(|metadata| metadata.span == *span))
                .map(|(name, _)| *name)
                .collect(),
            file_ids: self.file_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use foldhash::HashSet;
    use mago_allocator::LocalArena;
    use mago_database::file::File;
    use mago_database::file::FileType;
    use mago_names::resolver::NameResolver;
    use mago_php_version::PHPVersion;
    use mago_syntax::parser::parse_file;
    use mago_word::WordSet;
    use mago_word::word;

    use super::*;
    use crate::populator::populate_codebase;
    use crate::reference::SymbolReferences;
    use crate::scanner::scan_program;
    use crate::signature_builder;

    fn scan(name: &'static [u8], kind: FileType, source: &str) -> CodebaseMetadata {
        let file = File::new(Cow::Borrowed(name), kind, None, Cow::Owned(source.as_bytes().to_vec()));
        let arena = LocalArena::new();
        let program = parse_file(&arena, &file);
        assert!(!program.has_errors(), "{:?}", program.errors);
        let names = NameResolver::new(&arena).resolve(program);
        let mut metadata = scan_program(&arena, &file, program, &names, PHPVersion::LATEST);
        metadata.set_file_signature(file.id, signature_builder::build_file_signature(&file, program, &names));
        if kind.is_patch() {
            metadata.convert_partial_to_patch();
        }
        metadata
    }

    #[test]
    fn duplicate_snapshots_keep_only_the_winning_declaration_spans() {
        let source = concat!(
            "<?php\nclass Shared {} function value(): void {} const VALUE = 1; ",
            "class_alias(Shared::class, Alias::class);",
        );
        for (name, kind, prefix) in [
            (b"vendor.php".as_slice(), FileType::Vendored, ""),
            (b"host.php".as_slice(), FileType::Host, "// shifted declaration\n"),
        ] {
            let winner = scan(b"host.php", FileType::Host, source);
            let loser = scan(name, kind, &source.replacen("<?php\n", &format!("<?php\n{prefix}"), 1));
            let winner_spans = winner.entry_spans();
            let loser_spans = loser.entry_spans();
            let mut merged = CodebaseMetadata::new();
            merged.extend(loser);
            merged.extend(winner);
            populate_codebase(&mut merged, &mut SymbolReferences::new(), WordSet::default(), HashSet::default());

            let winner_keys = winner_spans.into_owned_keys(&merged);
            let loser_keys = loser_spans.into_owned_keys(&merged);
            assert_eq!(winner_keys.class_like_names.len(), 1);
            assert_eq!(winner_keys.function_like_keys.len(), 1);
            assert_eq!(winner_keys.constant_names.len(), 1);
            assert_eq!(winner_keys.class_like_aliases.len(), 1);
            assert!(loser_keys.class_like_names.is_empty());
            assert!(loser_keys.function_like_keys.is_empty());
            assert!(loser_keys.constant_names.is_empty());
            assert!(loser_keys.class_like_aliases.is_empty());
            assert_eq!(loser_keys.file_ids, vec![FileId::new(name)]);
            merged.remove_entries_by_keys(&loser_keys);
            assert_eq!(merged.class_likes.len(), 1);
            assert_eq!(merged.function_likes.len(), 1);
            assert_eq!(merged.constants.len(), 1);
            assert_eq!(merged.class_like_alias_declarations().count(), 1);
        }
    }

    #[test]
    fn snapshots_keep_alias_targets_and_patch_file_ownership() {
        let vendor = scan(
            b"vendor.php",
            FileType::Vendored,
            "<?php class Original {} function value(): mixed {} const VALUE = 1;",
        );
        let aliases = scan(
            b"aliases.php",
            FileType::Host,
            "<?php class_alias(Original::class, Middle::class); class_alias(Middle::class, Alias::class);",
        );
        let patch =
            scan(b"patch.php", FileType::Patch, "<?php class Original {} function value(): string {} const VALUE = 1;");
        let vendor_spans = vendor.entry_spans();
        let alias_spans = aliases.entry_spans();
        let patch_spans = patch.entry_spans();
        let mut merged = CodebaseMetadata::new();
        merged.extend(vendor);
        merged.extend(aliases);
        merged.extend(patch);
        merged.apply_patches_pass();
        populate_codebase(&mut merged, &mut SymbolReferences::new(), WordSet::default(), HashSet::default());

        let vendor_keys = vendor_spans.into_owned_keys(&merged);
        let alias_keys = alias_spans.into_owned_keys(&merged);
        let patch_keys = patch_spans.into_owned_keys(&merged);
        assert_eq!(merged.class_like_aliases[&word("alias")], word("original"));
        assert!(
            alias_keys
                .class_like_aliases
                .iter()
                .any(|(alias, target, _)| { *alias == word("alias") && *target == word("middle") })
        );
        assert_eq!(vendor_keys.function_like_keys.len(), 1);
        assert!(patch_keys.class_like_names.is_empty());
        assert!(patch_keys.function_like_keys.is_empty());
        assert!(patch_keys.constant_names.is_empty());
        assert_eq!(patch_keys.file_ids, vec![FileId::new(b"patch.php")]);
        assert_eq!(merged.patch_class_likes.len(), 1);
        assert_eq!(merged.patch_function_likes.len(), 1);
        assert_eq!(merged.patch_constants.len(), 1);
        merged.remove_entries_by_keys(&patch_keys);
        assert!(merged.patch_class_likes.is_empty());
        assert!(merged.patch_function_likes.is_empty());
        assert!(merged.patch_constants.is_empty());
        assert_eq!(merged.class_likes.len(), 1);
        assert_eq!(merged.function_likes.len(), 1);
        assert_eq!(merged.constants.len(), 1);
        merged.remove_entries_by_keys(&vendor_keys);
        assert!(merged.class_likes.is_empty());
        assert!(merged.function_likes.is_empty());
        assert!(merged.constants.is_empty());
    }
}
