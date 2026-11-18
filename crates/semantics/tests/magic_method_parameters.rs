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
fn magic_method_with_wrong_parameters_is_rejected() {
    for method in [
        "public function __clone($x) {}",
        "public function __clone($x = 1) {}",
        "public function __CLONE($x) {}",
        "public function __destruct($x) {}",
        "public function __destruct(?int $x = null) {}",
        "public function __get(...$n) {}",
        "public function __set($n, ...$v) {}",
        "public function __get(&$n) {}",
        "public function __get(&$n, $x) {}",
        "public function __set($n, &$v) {}",
        "public function __set(&$n, &$v) {}",
        "public function __isset(&$n) {}",
        "public function __unset(&$n) {}",
        "public function __call(&$n, $a) {}",
        "public static function __callStatic($n, &$a) {}",
        "public function __unserialize(array &$d) {}",
        "public static function __set_state(&$a) {}",
    ] {
        assert_eq!(count_semantics(&format!("<?php class A {{ {method} }}")), 1, "{method}");
    }
}

#[test]
fn magic_method_with_matching_parameters_is_accepted() {
    for method in [
        "public function __clone() {}",
        "public function __destruct() {}",
        "public function __clone(...$x) {}",
        "public function __destruct(...$x) {}",
        "public function __toString(...$x) {}",
        "public function __get($n, ...$rest) {}",
        "public function __get($n, &...$rest) {}",
        "public function __set($n, $v, ...$rest) {}",
        "public static function __callStatic($n, $a, ...$rest) {}",
        "public function __construct(&$x) {}",
        "public function __invoke(&$x) {}",
    ] {
        assert_eq!(count_semantics(&format!("<?php class A {{ {method} }}")), 0, "{method}");
    }
}
