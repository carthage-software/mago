use std::hash::BuildHasher;
use std::hash::Hasher;

use foldhash::fast::FixedState;

use mago_names::ResolvedNames;
use mago_syntax::ast::Trivia;

pub mod access;
pub mod argument;
pub mod array;
pub mod assignment;
pub mod attribute;
pub mod binary;
pub mod block;
pub mod call;
pub mod class_like;
pub mod clone;
pub mod conditional;
pub mod constant;
pub mod construct;
pub mod control_flow;
pub mod declare;
pub mod echo;
pub mod expression;
pub mod function_like;
pub mod global;
pub mod goto;
pub mod halt_compiler;
pub mod identifier;
pub mod inline;
pub mod instantiation;
pub mod keyword;
pub mod literal;
pub mod r#loop;
pub mod magic_constant;
pub mod modifier;
pub mod namespace;
pub mod partial_application;
pub mod pipe;
pub mod program;
pub mod r#return;
pub mod statement;
pub mod r#static;
pub mod string;
pub mod tag;
pub mod terminator;
pub mod throw;
pub mod r#try;
pub mod type_hint;
pub mod unary;
pub mod unset;
pub mod r#use;
pub mod variable;
pub mod r#yield;

const DEFAULT_IMPORTANT_COMMENT_PATTERNS: &[&[u8]] = &[b"@mago-", b"@"];

/// Hashes a byte slice into `hasher` after ASCII case-folding.
///
/// Used everywhere a PHP identifier is fingerprinted: PHP's case-insensitivity is ASCII-only,
/// so two identifiers that differ only in ASCII case must produce identical fingerprints.
/// Prefixed with the length to prevent `"ab"||"c"` colliding with `"a"||"bc"` when sibling
/// fields are absorbed back-to-back into the same hasher.
#[inline]
pub fn hash_ascii_lowercase<H>(bytes: &[u8], hasher: &mut H)
where
    H: std::hash::Hasher,
{
    hasher.write_usize(bytes.len());
    for &b in bytes {
        hasher.write_u8(b.to_ascii_lowercase());
    }
}

pub trait Fingerprintable {
    #[inline]
    fn fingerprint(&self, resolved_names: &ResolvedNames, options: &FingerprintOptions<'_>) -> u64 {
        let mut hasher = FixedState::default().build_hasher();
        self.fingerprint_with_hasher(&mut hasher, resolved_names, options);
        hasher.finish()
    }

    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher;
}

impl<T: Fingerprintable> Fingerprintable for Option<T> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        if let Some(value) = self {
            value.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl<T> Fingerprintable for &T
where
    T: Fingerprintable,
{
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        (*self).fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FingerprintOptions<'opts> {
    pub include_use_statements: bool,
    pub important_comment_patterns: &'opts [&'opts [u8]],
    pub signature_only: bool,
    pub trivia_context: Option<&'opts [Trivia<'opts>]>,
}

impl Default for FingerprintOptions<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            include_use_statements: false,
            important_comment_patterns: DEFAULT_IMPORTANT_COMMENT_PATTERNS,
            signature_only: false,
            trivia_context: None,
        }
    }
}

impl<'opts> FingerprintOptions<'opts> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub fn strict() -> Self {
        Self {
            include_use_statements: true,
            important_comment_patterns: &[],
            signature_only: false,
            trivia_context: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_trivia_context(mut self, trivia: &'opts [Trivia<'opts>]) -> Self {
        self.trivia_context = Some(trivia);
        self
    }

    #[inline]
    #[must_use]
    pub fn with_use_statements(mut self, include: bool) -> Self {
        self.include_use_statements = include;
        self
    }

    #[inline]
    #[must_use]
    pub fn with_comment_patterns(mut self, patterns: &'opts [&'opts [u8]]) -> Self {
        self.important_comment_patterns = patterns;
        self
    }

    #[inline]
    #[must_use]
    pub fn is_important_comment(&self, comment: &[u8]) -> bool {
        for pattern in self.important_comment_patterns {
            if memchr::memmem::find(comment, pattern).is_some() {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use mago_allocator::LocalArena;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_syntax::parser::parse_file;
    use std::borrow::Cow;
    use std::hash::Hasher;

    use super::*;

    pub(crate) fn fingerprint_code(code: &'static str) -> u64 {
        let arena = LocalArena::new();
        let file = File::ephemeral(Cow::Borrowed(b"code.php"), Cow::Borrowed(code.as_bytes()));
        let program = parse_file(&arena, &file);
        assert!(!program.has_errors(), "Failed to parse code, errors: {:?}", program.errors);
        let resolved_names = NameResolver::new(&arena).resolve(program);
        let options = FingerprintOptions::default();

        let mut hasher = foldhash::fast::FixedState::default().build_hasher();
        program.fingerprint_with_hasher(&mut hasher, &resolved_names, &options);
        hasher.finish()
    }

    #[test]
    fn test_important_comment_detection() {
        let opts = FingerprintOptions::default();

        assert!(opts.is_important_comment(b"// @mago-ignore"));
        assert!(opts.is_important_comment(b"/** @return string */"));
        assert!(!opts.is_important_comment(b"// Regular comment"));
        assert!(!opts.is_important_comment(b"/* Block comment */"));
    }

    #[test]
    fn test_use_statement() {
        let fp1 = fingerprint_code(indoc! {"
            <?php

            use Foo\\Bar;

            $_ = new Bar();
        "});

        let fp2 = fingerprint_code(indoc! {"
            <?php

            $_ = new \\Foo\\Bar;
        "});

        let fp3 = fingerprint_code(indoc! {"
            <?php

            use Foo\\Bar; // Brrrr

            $_ = new \\Foo\\Bar;
        "});

        let fp4 = fingerprint_code(indoc! {"
            <?php

            # Some comment
            $_ = new Foo\\Bar();
        "});

        assert_eq!(fp1, fp2);
        assert_eq!(fp1, fp3);
        assert_eq!(fp1, fp4);
    }

    #[test]
    fn test_docblock_comments_included() {
        let code_with_doc = "<?php\n/** @return string */\nfunction foo() { return 'x'; }";
        let code_without_doc = "<?php\nfunction foo() { return 'x'; }";

        let fp1 = fingerprint_code(code_with_doc);
        let fp2 = fingerprint_code(code_without_doc);

        assert_ne!(fp1, fp2, "docblock comments with @ should change fingerprint");
    }
}
