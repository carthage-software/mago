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
fn set_hook_returning_a_value_is_rejected() {
    assert_eq!(count_semantics("<?php class A { public int $a { set { return 1; } } }"), 1);
    assert_eq!(count_semantics("<?php class A { public $a { set { if ($value) { return $value; } } } }"), 1);
}

#[test]
fn get_hook_of_typed_property_returning_nothing_is_rejected() {
    assert_eq!(count_semantics("<?php class A { public int $a { get { return; } } }"), 1);
}

#[test]
fn hook_returns_php_accepts_are_allowed() {
    assert_eq!(
        count_semantics("<?php class A { public int $a { set { if ($value < 0) { return; } $this->a = $value; } } }"),
        0
    );
    assert_eq!(count_semantics("<?php class A { public $a { get { return; } } }"), 0);
    assert_eq!(
        count_semantics("<?php class A { public int $a { get { $f = function () { return; }; return 1; } } }"),
        0
    );
}
