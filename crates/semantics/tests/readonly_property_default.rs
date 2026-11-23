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

const READONLY_DEFAULTS: [&str; 10] = [
    "class A { public readonly int $x = 1; }",
    "class A { readonly int $x = 1; }",
    "class A { public readonly ?int $x = null; }",
    "trait T { public readonly int $x = 1; }",
    "readonly class A { public int $x = 1; }",
    "readonly class A { protected array $x = []; }",
    "final readonly class A { public int $x = 1; }",
    "readonly class A { public readonly int $x = 1; }",
    "abstract readonly class A { private ?string $x = null; }",
    "$a = new readonly class { public int $x = 1; };",
];

#[test]
fn readonly_property_default_is_rejected_before_php_8_6() {
    for source in READONLY_DEFAULTS {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP85), 1, "{source}");
    }
}

#[test]
fn readonly_property_default_is_accepted_from_php_8_6() {
    for source in READONLY_DEFAULTS {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP86), 0, "{source}");
    }
}

#[test]
fn property_without_readonly_default_is_accepted() {
    for source in [
        "class A { public readonly int $x; }",
        "class A { public int $x = 1; public static int $y = 2; }",
        "class A { public function __construct(public readonly int $x = 1) {} }",
        "readonly class A { public int $x; public function __construct(public int $y = 1) {} }",
        "$a = new readonly class { public int $x; };",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP85), 0, "{source}");
    }
}
