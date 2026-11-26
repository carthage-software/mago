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

const REJECTED: [&str; 14] = [
    "<?php #[\\NoDiscard] function f(): void {}",
    "<?php #[\\NoDiscard] function f(): never { throw new Exception(); }",
    "<?php class C { #[\\NoDiscard] public function m(): void {} }",
    "<?php class C { #[\\NoDiscard] public static function m(): never { exit; } }",
    "<?php interface I { #[\\NoDiscard] public function m(): void; }",
    "<?php abstract class A { #[\\NoDiscard] abstract public function m(): void; }",
    "<?php trait T { #[\\NoDiscard] public function m(): void {} }",
    "<?php enum E { case A; #[\\NoDiscard] public function m(): never { throw new Exception(); } }",
    "<?php $o = new class { #[\\NoDiscard] public function m(): void {} };",
    "<?php $f = #[\\NoDiscard] function (): void {};",
    "<?php $f = #[\\NoDiscard] fn(): never => throw new Exception();",
    "<?php namespace App; use NoDiscard; #[NoDiscard] function f(): void {}",
    "<?php use NoDiscard as Important; #[Important] function f(): void {}",
    "<?php #[\\NODISCARD('use it')] function f(): void {}",
];

#[test]
fn no_discard_on_void_or_never_is_rejected() {
    for source in REJECTED {
        assert_eq!(count_semantics(source, PHPVersion::PHP85), 1, "{source}");
    }
}

#[test]
fn no_discard_on_void_or_never_is_accepted_before_php_8_5() {
    for source in REJECTED {
        assert_eq!(count_semantics(source, PHPVersion::PHP84), 0, "{source}");
    }
}

#[test]
fn no_discard_with_a_return_value_is_accepted() {
    for source in [
        "<?php #[\\NoDiscard] function f(): int { return 1; }",
        "<?php #[\\NoDiscard] function f(): ?int { return null; }",
        "<?php #[\\NoDiscard] function f() {}",
        "<?php /** @return void */ #[\\NoDiscard] function f() {}",
        "<?php namespace App; #[NoDiscard] function f(): void {}",
        "<?php #[\\Deprecated] function f(): void {}",
        "<?php class C { #[\\NoDiscard] public function m(): static { return $this; } }",
    ] {
        assert_eq!(count_semantics(source, PHPVersion::PHP85), 0, "{source}");
    }
}
