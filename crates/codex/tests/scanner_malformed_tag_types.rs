#![allow(clippy::unwrap_used)]

use std::borrow::Cow;
use std::collections::BTreeMap;

use mago_allocator::LocalArena;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::scanner::scan_program;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file;

fn scan(source: &str) -> CodebaseMetadata {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"malformed.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "{:?}", program.errors);
    let names = NameResolver::new(&arena).resolve(program);

    scan_program(&arena, &file, program, &names, PHPVersion::LATEST)
}

fn issue_codes(mut metadata: CodebaseMetadata) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for issue in metadata.take_issues(true) {
        let Some(code) = issue.code else {
            panic!("scanning issue should have a code");
        };

        *counts.entry(code).or_insert(0) += 1;
    }

    counts
}

#[test]
fn unresolvable_variable_in_param_tag_is_reported() {
    let metadata = scan("<?php\n/** @param $unknown $x */\nfunction f(mixed $x): void {}\n");

    assert_eq!(issue_codes(metadata).get("invalid-param-tag"), Some(&1));
}

#[test]
fn parameter_link_in_param_tag_is_kept() {
    let metadata =
        scan("<?php\n/** @param $prototype $value */\nfunction f(mixed $prototype, mixed $value): void {}\n");

    assert!(issue_codes(metadata).is_empty());
}

#[test]
fn unresolvable_variable_as_return_tag_type_is_reported() {
    let metadata = scan("<?php\n/** @return $mixed */\nfunction g(): mixed { return 1; }\n");

    assert_eq!(issue_codes(metadata).get("invalid-return-tag"), Some(&1));
}

#[test]
fn parameter_link_as_return_tag_type_is_kept() {
    let metadata = scan("<?php\n/** @return $value */\nfunction g(mixed $value): mixed { return $value; }\n");

    assert!(issue_codes(metadata).is_empty());
}

#[test]
fn variable_as_property_var_tag_type_is_reported() {
    let metadata = scan("<?php\nclass C {\n/** @var $v */\npublic $v;\n}\n");

    assert_eq!(issue_codes(metadata).get("invalid-var-tag"), Some(&1));
}

#[test]
fn abutting_bracket_after_return_type_is_reported() {
    let metadata = scan("<?php\n/** @return array{a: string}> */\nfunction h(): array { return ['a' => 'b']; }\n");
    let codes = issue_codes(metadata);

    assert_eq!(codes.get("malformed-docblock-comment"), Some(&1));
    assert!(!codes.contains_key("invalid-return-tag"));
}

#[test]
fn well_formed_tags_are_silent() {
    let metadata = scan(
        "<?php\n/** @param int|string $x */\nfunction f(int|string $x): void {}\nclass C {\n/** @return $this */\npublic function fluent(): self { return $this; }\n}\n",
    );

    assert!(issue_codes(metadata).is_empty());
}
