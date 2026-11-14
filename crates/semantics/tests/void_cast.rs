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
fn void_cast_is_accepted_as_a_statement() {
    assert_eq!(count_semantics("<?php (void) strlen('abc');"), 0);
}

#[test]
fn void_cast_is_accepted_in_for_expressions() {
    assert_eq!(count_semantics("<?php for ((void) f(); (void) g(); (void) h()) {}"), 0);
}

#[test]
fn void_cast_is_rejected_as_a_value() {
    assert_eq!(count_semantics("<?php $length = (void) strlen('abc');"), 1);
    assert_eq!(count_semantics("<?php $callback = static fn(string $s) => (void) strlen($s);"), 1);
    assert_eq!(count_semantics("<?php (void) (void) f();"), 1);
}
