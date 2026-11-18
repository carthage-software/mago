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
fn callable_property_type_is_rejected() {
    for source in [
        "class A { public callable $x; }",
        "class A { public CALLABLE $x; }",
        "class A { public ?callable $x = null; }",
        "class A { public callable|string $x = 'f'; }",
        "class A { public string|callable $x = 'f'; }",
        "interface I {} interface J {} class A { public (I&J)|callable $x; }",
        "class A { public static callable $x; }",
        "class A { var callable $x; }",
        "class A { public callable $a, $b; }",
        "class A { public callable $x { get => 'strlen'; } }",
        "interface I { public callable $x { get; } }",
        "trait T { public callable $x; }",
        "$a = new class { public callable $x; };",
        "class A { public function __construct(public callable $x) {} }",
        "class A { public function __construct(protected ?callable $x = null) {} }",
        "class A { public function __construct(public callable|string $x) {} }",
        "class A { public function __construct(int $a, private(set) callable $x) {} }",
        "trait T { public function __construct(public callable $x) {} }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}")), 1, "{source}");
    }
}

#[test]
fn callable_outside_property_type_is_accepted() {
    for source in [
        "class A { public function __construct(callable $x) {} }",
        "class A { public function __construct(callable $a, public int $x) {} }",
        "class A { public function f(?callable $x): callable { return $x; } }",
        "function f(callable|string $x): ?callable { return null; }",
        "class A { public \\Closure $x; public \\Closure|string $y = 'f'; }",
        "class A { public function __construct(public \\Closure $x) {} }",
        "class A { /** @var callable */ public $x; }",
        "class A { public $callable; public function __construct(public int $isCallable = 0) {} }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}")), 0, "{source}");
    }
}
