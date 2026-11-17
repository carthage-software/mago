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
fn reserved_type_name_is_rejected_as_a_class_name() {
    for name in ["bool", "float", "int", "string", "void", "iterable", "object", "mixed"] {
        assert_eq!(count_semantics(&format!("<?php class {name} {{}}")), 1, "class {name}");
    }

    assert_eq!(count_semantics("<?php namespace App\\Support; final class Object {}"), 1);
}

#[test]
fn soft_reserved_name_is_accepted_as_a_class_name() {
    assert_eq!(count_semantics("<?php class Resource {}"), 0);
    assert_eq!(count_semantics("<?php class Numeric {}"), 0);
}
