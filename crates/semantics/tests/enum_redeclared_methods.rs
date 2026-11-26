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
    let issues = SemanticsChecker::new(PHPVersion::PHP81).check(&file, program, &names);

    issues.iter().filter(|issue| issue.code.as_deref() == Some("semantics")).count()
}

#[test]
fn enum_method_declared_by_php_is_rejected() {
    for source in [
        "<?php enum E { case A; public static function cases(): array { return []; } }",
        "<?php enum E: int { case A = 1; public static function cases(): array { return []; } }",
        "<?php enum E: string { case A = 'a'; public static function CASES(): array { return []; } }",
        "<?php enum E: int { case A = 1; public static function from(int|string $v): static { return self::A; } }",
        "<?php enum E: int { case A = 1; public static function tryFrom(int|string $v): ?static { return null; } }",
        "<?php enum E: string { case A = 'a'; public static function TRYFROM(int|string $v): ?static { return null; } }",
    ] {
        assert_eq!(count_semantics(source), 1, "{source}");
    }
}

#[test]
fn enum_method_not_declared_by_php_is_accepted() {
    for source in [
        "<?php enum E { case A; public static function from(int $v): self { return self::A; } }",
        "<?php enum E { case A; public static function tryFrom(int $v): ?self { return null; } }",
        "<?php enum E: int { case A = 1; public static function fromValue(int $v): self { return self::A; } }",
        "<?php trait T { public static function cases(): array { return []; } } enum E { use T; case A; }",
        "<?php interface I { public static function cases(): array; } enum E implements I { case A; }",
        "<?php class A { public static function cases(): array { return []; } public static function from($v) {} }",
    ] {
        assert_eq!(count_semantics(source), 0, "{source}");
    }
}
