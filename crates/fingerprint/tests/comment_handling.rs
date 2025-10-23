use bumpalo::Bump;

use mago_database::file::File;
use mago_fingerprint::FingerprintOptions;
use mago_fingerprint::Fingerprintable;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

fn get_fingerprint(code: &'static str) -> u64 {
    let arena = Bump::new();
    let file = File::ephemeral("test.php".into(), code.into());
    let (program, error) = parse_file(&arena, &file);
    assert!(error.is_none(), "Parse error: {:?}", error);

    let resolved_names = NameResolver::new(&arena).resolve(program);
    let options = FingerprintOptions::default();

    program.fingerprint(&resolved_names, &options)
}

fn get_fingerprint_with_options(code: &'static str, options: FingerprintOptions) -> u64 {
    let arena = Bump::new();
    let file = File::ephemeral("test.php".into(), code.into());
    let (program, error) = parse_file(&arena, &file);
    assert!(error.is_none(), "Parse error: {:?}", error);

    let resolved_names = NameResolver::new(&arena).resolve(program);

    program.fingerprint(&resolved_names, &options)
}

#[test]
fn test_single_line_comments_ignored() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_comment = get_fingerprint("<?php // This is a comment\nclass Foo {}");

    assert_eq!(without, with_comment, "Single-line comments should be ignored");
}

#[test]
fn test_block_comments_ignored() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_comment = get_fingerprint("<?php /* Block comment */ class Foo {}");

    assert_eq!(without, with_comment, "Block comments should be ignored");
}

#[test]
fn test_hash_comments_ignored() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_comment = get_fingerprint("<?php # Hash comment\nclass Foo {}");

    assert_eq!(without, with_comment, "Hash comments should be ignored");
}

#[test]
fn test_multiline_block_comments_ignored() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_comment = get_fingerprint(
        "<?php
        /*
         * This is a multiline
         * block comment
         */
        class Foo {}",
    );

    assert_eq!(without, with_comment, "Multiline block comments should be ignored");
}

#[test]
fn test_inline_comments_ignored() {
    let without = get_fingerprint("<?php function foo() { return 1; }");
    let with_comment = get_fingerprint("<?php function foo() { return 1; /* inline */ }");

    assert_eq!(without, with_comment, "Inline comments should be ignored");
}

#[test]
fn test_multiple_regular_comments_ignored() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_comments = get_fingerprint(
        "<?php
        // Comment 1
        /* Comment 2 */
        # Comment 3
        class Foo {}",
    );

    assert_eq!(without, with_comments, "Multiple regular comments should be ignored");
}

#[test]
fn test_docblock_with_at_symbol_included() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_docblock = get_fingerprint("<?php /** @return string */ class Foo {}");

    assert_ne!(without, with_docblock, "Docblocks with @ should be included");
}

#[test]
fn test_docblock_return_type_change() {
    let return_string = get_fingerprint("<?php /** @return string */ function foo() {}");
    let return_int = get_fingerprint("<?php /** @return int */ function foo() {}");

    assert_ne!(return_string, return_int, "Docblock content changes should change fingerprint");
}

#[test]
fn test_docblock_param_addition() {
    let without_param = get_fingerprint("<?php /** @return void */ function foo() {}");
    let with_param = get_fingerprint("<?php /** @param int $x\n@return void */ function foo() {}");

    assert_ne!(without_param, with_param, "Adding docblock param should change fingerprint");
}

#[test]
fn test_docblock_on_class() {
    let without = get_fingerprint("<?php class Foo {}");
    let with = get_fingerprint("<?php /** @author John Doe */ class Foo {}");

    assert_ne!(without, with, "Docblock on class should change fingerprint");
}

#[test]
fn test_docblock_on_property() {
    let without = get_fingerprint("<?php class A { public $x; }");
    let with = get_fingerprint("<?php class A { /** @var string */ public $x; }");

    assert_ne!(without, with, "Docblock on property should change fingerprint");
}

#[test]
fn test_docblock_on_method() {
    let without = get_fingerprint("<?php class A { function foo() {} }");
    let with = get_fingerprint("<?php class A { /** @return mixed */ function foo() {} }");

    assert_ne!(without, with, "Docblock on method should change fingerprint");
}

#[test]
fn test_multiple_docblock_tags() {
    let one_tag = get_fingerprint("<?php /** @return string */ function foo() {}");
    let two_tags = get_fingerprint("<?php /** @param int $x\n@return string */ function foo() {}");

    assert_ne!(one_tag, two_tags, "Additional docblock tags should change fingerprint");
}

#[test]
fn test_complex_docblock() {
    let simple = get_fingerprint("<?php function foo() {}");
    let complex = get_fingerprint(
        "<?php
        /**
         * Complex function with detailed docs.
         *
         * @param string $name The user name
         * @param int $age The user age
         * @return array<string, mixed> User data
         * @throws InvalidArgumentException
         * @deprecated Use newFoo() instead
         */
        function foo() {}",
    );

    assert_ne!(simple, complex, "Complex docblock should change fingerprint");
}

#[test]
fn test_mago_ignore_directive() {
    let without = get_fingerprint("<?php function foo() {}");
    let with_ignore = get_fingerprint("<?php // @mago-ignore\nfunction foo() {}");

    assert_ne!(without, with_ignore, "@mago-ignore should be included");
}

#[test]
fn test_mago_expect_directive() {
    let without = get_fingerprint("<?php function foo() { $x = null; }");
    let with_expect = get_fingerprint("<?php function foo() { // @mago-expect mixed-type\n$x = null; }");

    assert_ne!(without, with_expect, "@mago-expect should be included");
}

#[test]
fn test_mago_baseline_directive() {
    let without = get_fingerprint("<?php function foo() {}");
    let with_baseline = get_fingerprint("<?php // @mago-baseline\nfunction foo() {}");

    assert_ne!(without, with_baseline, "@mago-baseline should be included");
}

#[test]
fn test_mago_directive_in_block_comment() {
    let without = get_fingerprint("<?php function foo() {}");
    let with_directive = get_fingerprint("<?php /* @mago-ignore */ function foo() {}");

    assert_ne!(without, with_directive, "Mago directive in block comment should be included");
}

#[test]
fn test_mago_directive_different_content() {
    let ignore = get_fingerprint("<?php // @mago-ignore\nfunction foo() {}");
    let expect = get_fingerprint("<?php // @mago-expect issue\nfunction foo() {}");

    assert_ne!(ignore, expect, "Different mago directives should differ");
}

#[test]
fn test_inline_var_type_hint() {
    let without = get_fingerprint("<?php function foo() { $x = 1; }");
    let with_hint = get_fingerprint("<?php function foo() { /* @var int $x */ $x = 1; }");

    assert_ne!(without, with_hint, "Inline @var should be included");
}

#[test]
fn test_inline_var_type_change() {
    let int_hint = get_fingerprint("<?php function foo() { /* @var int $x */ $x = 1; }");
    let string_hint = get_fingerprint("<?php function foo() { /* @var string $x */ $x = 1; }");

    assert_ne!(int_hint, string_hint, "Inline type hint change should change fingerprint");
}

#[test]
fn test_inline_var_generic_type() {
    let simple = get_fingerprint("<?php function foo() { /* @var array $x */ $x = []; }");
    let generic = get_fingerprint("<?php function foo() { /* @var array<string> $x */ $x = []; }");

    assert_ne!(simple, generic, "Generic type in @var should change fingerprint");
}

#[test]
fn test_custom_pattern_not_included_by_default() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_custom = get_fingerprint("<?php // @custom-directive\nclass Foo {}");

    assert_ne!(without, with_custom, "@ pattern in default options matches all @ symbols");
}

#[test]
fn test_custom_pattern_with_options() {
    let code = "<?php // #custom-directive\nclass Foo {}";
    let code_without = "<?php class Foo {}";

    let default_options = FingerprintOptions::default();
    let custom_options = FingerprintOptions::default().with_comment_patterns(&["#custom-"]);

    let fp_default = get_fingerprint_with_options(code, default_options);
    let fp_custom = get_fingerprint_with_options(code, custom_options);
    let fp_without = get_fingerprint(code_without);

    assert_eq!(fp_default, fp_without, "Default should ignore # pattern");
    assert_ne!(fp_custom, fp_without, "Custom options should include # pattern");
}

#[test]
fn test_multiple_custom_patterns() {
    let code1 = "<?php // @pattern1\nclass Foo {}";
    let code2 = "<?php // @pattern2\nclass Foo {}";
    let code_without = "<?php class Foo {}";

    let options = FingerprintOptions::default().with_comment_patterns(&["@pattern1", "@pattern2"]);

    let fp1 = get_fingerprint_with_options(code1, options);
    let fp2 = get_fingerprint_with_options(code2, options);
    let fp_without = get_fingerprint(code_without);

    assert_ne!(fp1, fp_without, "Pattern1 should be included");
    assert_ne!(fp2, fp_without, "Pattern2 should be included");
    assert_ne!(fp1, fp2, "Different patterns should differ");
}

#[test]
fn test_empty_pattern_list() {
    let code = "<?php /** @return string */ function foo() {}";
    let code_without = "<?php function foo() {}";

    let options = FingerprintOptions::default().with_comment_patterns(&[]);

    let fp_with_patterns = get_fingerprint_with_options(code, options);
    let fp_without = get_fingerprint(code_without);

    assert_eq!(fp_with_patterns, fp_without, "Empty pattern list should ignore all comments");
}

#[test]
fn test_comment_before_statement() {
    let without = get_fingerprint("<?php function foo() { return 1; }");
    let with = get_fingerprint("<?php function foo() { /** @return int */ return 1; }");

    assert_ne!(without, with, "Comment before statement should be included if important");
}

#[test]
fn test_comment_after_statement() {
    let without = get_fingerprint("<?php function foo() { return 1; }");
    let with = get_fingerprint("<?php function foo() { return 1; /* regular comment */ }");

    assert_eq!(without, with, "Regular comment after statement should be ignored");
}

#[test]
fn test_comment_between_statements() {
    let without = get_fingerprint("<?php function foo() { $x = 1; $y = 2; }");
    let with_regular = get_fingerprint("<?php function foo() { $x = 1; // comment\n$y = 2; }");
    let with_docblock = get_fingerprint("<?php function foo() { $x = 1; /* @var int */ $y = 2; }");

    assert_eq!(without, with_regular, "Regular comment between statements should be ignored");
    assert_ne!(without, with_docblock, "Important comment between statements should be included");
}

#[test]
fn test_docblock_whitespace_normalization() {
    let compact = get_fingerprint("<?php /** @return string */ class Foo {}");
    let spaced = get_fingerprint("<?php /**  @return  string  */ class Foo {}");

    assert_ne!(compact, spaced, "Whitespace in docblocks is preserved");
}

#[test]
fn test_docblock_newline_formatting() {
    let single_line = get_fingerprint("<?php /** @return string */ function foo() {}");
    let multi_line = get_fingerprint(
        "<?php
        /**
         * @return string
         */
        function foo() {}",
    );

    assert_ne!(single_line, multi_line, "Docblock formatting affects fingerprint");
}

#[test]
fn test_regular_and_important_comments_mixed() {
    let base = get_fingerprint("<?php class Foo {}");
    let with_all = get_fingerprint(
        "<?php
        // Regular comment (ignored)
        /** @template T */
        /* @var array<int> */
        // @mago-ignore
        // Another regular comment
        class Foo {}",
    );

    assert_ne!(base, with_all, "Mix of comments should include only important ones");
}

#[test]
fn test_important_comments_in_class_body() {
    let without = get_fingerprint("<?php class A { public $x; }");
    let with = get_fingerprint(
        "<?php
        class A {
            /** @var string */
            public $x;

            // Regular comment

            /** @return void */
            public function foo() {}
        }",
    );

    assert_ne!(without, with, "Important comments in class body should be included");
}

#[test]
fn test_empty_docblock() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_empty = get_fingerprint("<?php /** */ class Foo {}");

    assert_eq!(without, with_empty, "Empty docblock without @ should be ignored");
}

#[test]
fn test_docblock_without_tags() {
    let without = get_fingerprint("<?php class Foo {}");
    let with_description = get_fingerprint("<?php /** This is a description */ class Foo {}");

    assert_eq!(without, with_description, "Docblock without @ should be ignored");
}

#[test]
fn test_at_symbol_in_string() {
    let without = get_fingerprint("<?php function foo() { $x = 'test'; }");
    let with_at = get_fingerprint("<?php function foo() { $x = '@test'; }");

    assert_ne!(without, with_at, "@ in string changes code semantics");
}

#[test]
fn test_comment_in_string() {
    let str1 = get_fingerprint("<?php $x = 'test';");
    let str2 = get_fingerprint("<?php $x = 'test // comment';");

    assert_ne!(str1, str2, "Comment-like text in string is part of value");
}

#[test]
fn test_nested_comment_markers() {
    let fp = get_fingerprint("<?php /* This has asterisks * and * markers */ class Foo {}");
    assert_ne!(fp, 0, "Comments with asterisks should parse");
}

#[test]
fn test_unclosed_comment_in_valid_code() {
    let fp = get_fingerprint("<?php /* comment */ class Foo {}");
    assert_ne!(fp, 0, "Should handle comments correctly");
}

#[test]
fn test_param_tag() {
    let without = get_fingerprint("<?php function foo() {}");
    let with = get_fingerprint("<?php /** @param string $name */ function foo() {}");

    assert_ne!(without, with, "@param tag should be included");
}

#[test]
fn test_return_tag() {
    let without = get_fingerprint("<?php function foo() {}");
    let with = get_fingerprint("<?php /** @return void */ function foo() {}");

    assert_ne!(without, with, "@return tag should be included");
}

#[test]
fn test_throws_tag() {
    let without = get_fingerprint("<?php function foo() {}");
    let with = get_fingerprint("<?php /** @throws Exception */ function foo() {}");

    assert_ne!(without, with, "@throws tag should be included");
}

#[test]
fn test_var_tag() {
    let without = get_fingerprint("<?php class A { public $x; }");
    let with = get_fingerprint("<?php class A { /** @var string */ public $x; }");

    assert_ne!(without, with, "@var tag should be included");
}

#[test]
fn test_deprecated_tag() {
    let without = get_fingerprint("<?php function foo() {}");
    let with = get_fingerprint("<?php /** @deprecated */ function foo() {}");

    assert_ne!(without, with, "@deprecated tag should be included");
}

#[test]
fn test_template_tag() {
    let without = get_fingerprint("<?php class Foo {}");
    let with = get_fingerprint("<?php /** @template T */ class Foo {}");

    assert_ne!(without, with, "@template tag should be included");
}

#[test]
fn test_extends_tag() {
    let without = get_fingerprint("<?php class Foo {}");
    let with = get_fingerprint("<?php /** @extends BaseClass<T> */ class Foo {}");

    assert_ne!(without, with, "@extends tag should be included");
}

#[test]
fn test_implements_tag() {
    let without = get_fingerprint("<?php class Foo {}");
    let with = get_fingerprint("<?php /** @implements Iterator<int, string> */ class Foo {}");

    assert_ne!(without, with, "@implements tag should be included");
}

#[test]
fn test_adding_comment_preserves_structure() {
    let original = get_fingerprint("<?php class Foo { public function bar() { return 1; } }");
    let with_comments = get_fingerprint(
        "<?php
        // Comment
        class Foo {
            // Another comment
            public function bar() {
                // Yet another
                return 1;
            }
        }",
    );

    assert_eq!(original, with_comments, "Regular comments don't change structure");
}

#[test]
fn test_important_comments_affect_structure() {
    let original = get_fingerprint("<?php class Foo { public function bar() { return 1; } }");
    let with_docblocks = get_fingerprint(
        "<?php
        /** @template T */
        class Foo {
            /** @return int */
            public function bar() {
                return 1;
            }
        }",
    );

    assert_ne!(original, with_docblocks, "Important comments change fingerprint");
}
