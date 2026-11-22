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
fn variadic_promoted_property_is_rejected() {
    for source in [
        "class A { public function __construct(public int ...$x) {} }",
        "class A { public function __construct(private ...$x) {} }",
        "class A { public function __construct(protected ?string ...$x) {} }",
        "class A { public function __construct(public int &...$x) {} }",
        "class A { public function __construct(public int $a, public int ...$b) {} }",
        "class A { public function __CONSTRUCT(public int ...$x) {} }",
        "trait T { public function __construct(public int ...$x) {} }",
        "$a = new class { public function __construct(public int ...$x) {} };",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP80), 1, "{source}");
    }
}

#[test]
fn variadic_promoted_property_with_newer_modifiers_is_rejected() {
    for source in [
        "class A { public function __construct(readonly int ...$x) {} }",
        "class A { public function __construct(public readonly int ...$x) {} }",
        "class A { public function __construct(public private(set) int ...$x) {} }",
        "class A { public function __construct(private(set) int ...$x) {} }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP84), 1, "{source}");
    }
}

#[test]
fn variadic_parameter_without_promotion_is_accepted() {
    for source in [
        "class A { public function __construct(int ...$x) {} }",
        "class A { public function __construct(public int $a, int ...$b) {} }",
        "function f(int ...$x) {}",
        "$f = function (int ...$x) {};",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP80), 0, "{source}");
    }
}
