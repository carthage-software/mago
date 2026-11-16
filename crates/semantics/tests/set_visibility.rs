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
fn set_visibility_wider_than_read_visibility_is_rejected() {
    assert_eq!(count_semantics("<?php class A { protected public(set) int $a = 0; }"), 1);
    assert_eq!(count_semantics("<?php class A { private protected(set) int $a = 0; }"), 1);
    assert_eq!(count_semantics("<?php class A { protected public(set) readonly int $a; }"), 1);
    assert_eq!(count_semantics("<?php class A { public function __construct(protected public(set) int $a) {} }"), 1);
}

#[test]
fn set_visibility_as_narrow_or_narrower_is_accepted() {
    assert_eq!(count_semantics("<?php class A { public private(set) int $a = 0; }"), 0);
    assert_eq!(count_semantics("<?php class A { protected private(set) int $a = 0; }"), 0);
    assert_eq!(count_semantics("<?php class A { public(set) int $a = 0; }"), 0);
    assert_eq!(count_semantics("<?php class A { public function __construct(public protected(set) int $a) {} }"), 0);
}
