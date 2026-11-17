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
fn parent_type_without_a_parent_is_rejected() {
    assert_eq!(count_semantics("<?php final class A { public function f(): parent { return $this; } }"), 1);
    assert_eq!(count_semantics("<?php class A { public function f(?parent $p): void {} }"), 1);
    assert_eq!(count_semantics("<?php class A { public parent|int $p = 1; }"), 1);
    assert_eq!(count_semantics("<?php interface I { public function f(parent $p); }"), 1);
    assert_eq!(count_semantics("<?php enum E { public function f(): ?parent { return null; } }"), 1);
    assert_eq!(count_semantics("<?php $a = new class { public function f(): parent { return $this; } };"), 1);
}

#[test]
fn parent_type_with_a_parent_or_in_a_trait_is_allowed() {
    assert_eq!(
        count_semantics("<?php class A {} class B extends A { public function f(): parent { return $this; } }"),
        0
    );
    assert_eq!(count_semantics("<?php trait T { public function f(): parent { return $this; } }"), 0);
    assert_eq!(count_semantics("<?php class A { public function f() { return fn(parent $p) => 1; } }"), 0);
}
