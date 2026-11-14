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
fn promoted_property_is_rejected_outside_a_constructor() {
    assert_eq!(count_semantics("<?php class A { public function __constructor(private int $x) {} }"), 1);
    assert_eq!(count_semantics("<?php class A { public function f(private int $x): void {} }"), 1);
}

#[test]
fn promoted_property_is_rejected_in_an_abstract_constructor() {
    assert_eq!(count_semantics("<?php abstract class A { abstract public function __construct(private int $x); }"), 1);
}

#[test]
fn promoted_property_is_accepted_in_a_constructor() {
    assert_eq!(count_semantics("<?php class A { public function __construct(private int $x) {} }"), 0);
    assert_eq!(count_semantics("<?php class A { public function __CONSTRUCT(private int $x) {} }"), 0);
}
