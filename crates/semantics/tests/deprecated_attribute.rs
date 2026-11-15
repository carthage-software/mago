use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

fn count_semantics(source: &str, version: PHPVersion) -> usize {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(program.errors.is_empty(), "test source did not parse: {:?}", program.errors);

    let names = NameResolver::new(&arena).resolve(program);
    let issues = SemanticsChecker::new(version).check(&file, program, &names);

    issues.iter().filter(|issue| issue.code.as_deref() == Some("semantics")).count()
}

#[test]
fn deprecated_attribute_on_a_class_like_is_rejected() {
    for source in [
        "<?php #[\\Deprecated] interface I {}",
        "<?php #[\\Deprecated] enum E {}",
        "<?php $a = new #[\\Deprecated] class {};",
        "<?php #[\\Attribute, \\deprecated] class A {}",
    ] {
        assert_eq!(count_semantics(source, PHPVersion::PHP85), 1, "{source}");
    }
}

#[test]
fn deprecated_attribute_on_a_trait_or_another_attribute_is_accepted() {
    assert_eq!(count_semantics("<?php #[\\Deprecated] trait T {}", PHPVersion::PHP85), 0);
    assert_eq!(count_semantics("<?php namespace N; #[Deprecated] class A {}", PHPVersion::PHP85), 0);
}

#[test]
fn deprecated_attribute_is_checked_from_php_84() {
    assert_eq!(count_semantics("<?php #[\\Deprecated] class A {}", PHPVersion::PHP83), 0);
    assert_eq!(count_semantics("<?php #[\\Deprecated] class A {}", PHPVersion::PHP84), 1);
}
