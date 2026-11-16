use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

fn count_semantics(source: &str) -> usize {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(program.errors.is_empty(), "test source did not parse: {:?}", program.errors);

    let names = NameResolver::new(&arena).resolve(program);
    let issues = SemanticsChecker::new(PHPVersion::PHP85).check(&file, program, &names);

    issues.iter().filter(|issue| issue.code.as_deref() == Some("semantics")).count()
}

fn count_for_parameter(hint: &str) -> usize {
    count_semantics(&format!("<?php namespace App; class Foo {{}} function f({hint} $x): void {{}}"))
}

#[test]
fn duplicate_types_are_rejected() {
    assert_eq!(count_for_parameter("string|string"), 1);
    assert_eq!(count_for_parameter("int|INT"), 1);
    assert_eq!(count_for_parameter("Foo|null|Foo"), 1);
    assert_eq!(count_for_parameter("Foo|\\App\\Foo"), 1);
    assert_eq!(count_for_parameter("Foo|foo"), 1);
    assert_eq!(count_for_parameter("float|int|float"), 1);
}

#[test]
fn types_covered_by_another_member_are_rejected() {
    assert_eq!(count_for_parameter("bool|false"), 1);
    assert_eq!(count_for_parameter("true|bool"), 1);
    assert_eq!(count_for_parameter("true|false"), 1);
    assert_eq!(count_for_parameter("iterable|array"), 1);
    assert_eq!(count_for_parameter("\\Traversable|iterable"), 1);
    assert_eq!(count_for_parameter("object|Foo"), 1);
    assert_eq!(count_for_parameter("(Foo&Bar)|Foo"), 1);
    assert_eq!(count_for_parameter("(Foo&Bar)|(Bar&Foo)"), 1);
}

#[test]
fn distinct_types_are_accepted() {
    assert_eq!(count_for_parameter("int|string|null"), 0);
    assert_eq!(count_for_parameter("\\Closure|callable"), 0);
    assert_eq!(count_for_parameter("object|null"), 0);
    assert_eq!(count_for_parameter("(Foo&Bar)|(Foo&Baz)|null"), 0);
    assert_eq!(count_for_parameter("Foo|\\Foo"), 0);
}
