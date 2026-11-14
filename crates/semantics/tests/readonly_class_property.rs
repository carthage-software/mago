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
fn readonly_class_property_is_checked_as_readonly() {
    assert_eq!(count_semantics("<?php readonly class A { public $x; }", PHPVersion::PHP82), 1);
    assert_eq!(count_semantics("<?php readonly class A { public readonly $x; }", PHPVersion::PHP82), 1);
    assert_eq!(count_semantics("<?php $a = new readonly class { public $x; };", PHPVersion::PHP83), 1);
    assert_eq!(count_semantics("<?php readonly class A { public static int $x; }", PHPVersion::PHP82), 1);
    assert_eq!(count_semantics("<?php readonly class A { public static readonly int $x; }", PHPVersion::PHP82), 1);
}

#[test]
fn typed_or_non_readonly_property_is_accepted() {
    assert_eq!(count_semantics("<?php readonly class A { public int $x; }", PHPVersion::PHP85), 0);
    assert_eq!(count_semantics("<?php class A { public $x; public static $y; }", PHPVersion::PHP85), 0);
}

#[test]
fn unsupported_readonly_class_reports_only_the_version() {
    assert_eq!(count_semantics("<?php readonly class A { public $x; public static int $y; }", PHPVersion::PHP81), 1);
    assert_eq!(
        count_semantics("<?php $a = new readonly class { public $x; public static int $y; };", PHPVersion::PHP82),
        1
    );
}
