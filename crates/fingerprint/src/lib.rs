use std::hash::Hasher;

use ahash::AHasher;

use mago_names::ResolvedNames;

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
pub mod closure_creation;
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

const DEFAULT_IMPORTANT_COMMENT_PATTERNS: &[&str] = &["@mago-", "@"];

pub trait Fingerprintable {
    fn fingerprint(&self, resolved_names: &ResolvedNames, options: &FingerprintOptions<'_>) -> u64 {
        let mut hasher = AHasher::default();
        self.fingerprint_with_hasher(&mut hasher, resolved_names, options);
        hasher.finish()
    }

    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    );
}

impl<T: Fingerprintable> Fingerprintable for Option<T> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        if let Some(value) = self {
            value.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl<T> Fingerprintable for &T
where
    T: Fingerprintable,
{
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        (*self).fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FingerprintOptions<'a> {
    pub include_use_statements: bool,
    pub important_comment_patterns: &'a [&'a str],
}

impl<'a> Default for FingerprintOptions<'a> {
    fn default() -> Self {
        Self { include_use_statements: false, important_comment_patterns: DEFAULT_IMPORTANT_COMMENT_PATTERNS }
    }
}

impl<'a> FingerprintOptions<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn strict() -> Self {
        Self { include_use_statements: true, important_comment_patterns: &[] }
    }

    pub fn with_use_statements(mut self, include: bool) -> Self {
        self.include_use_statements = include;
        self
    }

    pub fn with_comment_patterns(mut self, patterns: &'a [&'a str]) -> Self {
        self.important_comment_patterns = patterns;
        self
    }

    pub fn is_important_comment(&self, comment: &str) -> bool {
        for pattern in self.important_comment_patterns {
            if comment.contains(pattern) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use indoc::indoc;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_syntax::parser::parse_file;
    use std::hash::Hasher;

    use super::*;

    pub(crate) fn fingerprint_code(code: &'static str) -> u64 {
        let arena = Bump::new();
        let file = File::ephemeral("code.php".into(), code.into());
        let (program, _parse_error) = parse_file(&arena, &file);
        let resolved_names = NameResolver::new(&arena).resolve(program);
        let options = FingerprintOptions::default();

        let mut hasher = ahash::AHasher::default();
        program.fingerprint_with_hasher(&mut hasher, &resolved_names, &options);
        hasher.finish()
    }

    #[test]
    fn test_important_comment_detection() {
        let opts = FingerprintOptions::default();

        assert!(opts.is_important_comment("// @mago-ignore"));
        assert!(opts.is_important_comment("/** @return string */"));
        assert!(!opts.is_important_comment("// Regular comment"));
        assert!(!opts.is_important_comment("/* Block comment */"));
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
