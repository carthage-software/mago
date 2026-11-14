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
fn magic_method_with_a_wrong_declared_type_is_rejected() {
    for method in [
        "public function __clone(): int {}",
        "public function __set($n, $v): mixed {}",
        "public function __unset($n): int {}",
        "public function __isset($n): void {}",
        "public function __toString(): ?string {}",
        "public function __toString(): static {}",
        "public function __toString(): Stringable {}",
        "public function __toString(): (Stringable&Countable)|string {}",
        "public function __TOSTRING(): int {}",
        "public function __debugInfo(): array|false {}",
        "public function __serialize(): iterable {}",
        "public function __serialize(): array|Traversable {}",
        "public function __unserialize(array $d): int {}",
        "public static function __set_state(array $a): ?object {}",
        "public function __sleep(): string {}",
        "public function __wakeup(): int {}",
        "public function __get(int $n) {}",
        "public function __set(int $n, $v) {}",
        "public function __unset(int $n) {}",
        "public function __isset(int $n) {}",
        "public function __call(int $n, array $a) {}",
        "public function __call(string $n, string $a) {}",
        "public static function __callStatic(int $n, array $a) {}",
        "public static function __callStatic(string $n, callable $a) {}",
        "public function __unserialize(self $d) {}",
        "public static function __set_state(string $a) {}",
    ] {
        assert_eq!(count_semantics(&format!("<?php class A {{ {method} }}")), 1, "{method}");
    }
}

#[test]
fn magic_method_with_a_matching_declared_type_is_accepted() {
    for method in [
        "public function __toString(): string {}",
        "public function __toString(): never {}",
        "public function __clone(): void {}",
        "public function __isset($n): true {}",
        "public function __debugInfo(): ?array {}",
        "public function __debugInfo(): null {}",
        "public static function __set_state(array $a): static {}",
        "public static function __set_state(array $a): self {}",
        "public function __get(int|string $n) {}",
        "public function __get(mixed $n) {}",
        "public function __call(string $n, iterable $a) {}",
        "public function __get($n): int {}",
    ] {
        assert_eq!(count_semantics(&format!("<?php class A {{ {method} }}")), 0, "{method}");
    }
}
