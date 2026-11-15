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

#[test]
fn import_with_a_name_already_in_use_is_rejected() {
    for source in [
        "<?php use Foo\\X as A; use Bar\\Y as a;",
        "<?php use function Foo\\f; use function Bar\\F;",
        "<?php use function Foo\\{f, g}; use function Bar\\f;",
        "<?php use const Foo\\X; use const Bar\\X;",
        "<?php namespace N; use Foo\\A; use Bar\\A;",
    ] {
        assert_eq!(count_semantics(source), 1, "{source}");
    }
}

#[test]
fn import_with_a_free_name_is_accepted() {
    for source in [
        "<?php use const Foo\\X; use const Bar\\x;",
        "<?php use Foo\\A; use function Bar\\A; use const Baz\\A;",
        "<?php use Foo\\{A, function A, const A};",
    ] {
        assert_eq!(count_semantics(source), 0, "{source}");
    }
}
