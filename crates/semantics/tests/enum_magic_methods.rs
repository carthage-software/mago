use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

fn collect_codes(source: &str) -> Vec<String> {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(program.errors.is_empty(), "test source did not parse: {:?}", program.errors);

    let names = NameResolver::new(&arena).resolve(program);
    let issues = SemanticsChecker::new(PHPVersion::new(8, 4, 0)).check(&file, program, &names);

    issues.iter().filter_map(|issue| issue.code.clone()).collect()
}

fn assert_reports_semantics(source: &str) {
    let codes = collect_codes(source);
    assert!(codes.iter().any(|c| c == "semantics"), "expected a semantics issue, got {codes:?}");
}

fn assert_no_semantics(source: &str) {
    let codes = collect_codes(source);
    assert!(!codes.iter().any(|c| c == "semantics"), "did not expect a semantics issue, got {codes:?}");
}

#[test]
fn forbidden_magic_methods_in_enum_are_rejected() {
    assert_reports_semantics("<?php enum E { public function __construct() {} }");
    assert_reports_semantics("<?php enum E { public function __destruct() {} }");
    assert_reports_semantics("<?php enum E { public function __clone(): void {} }");
    assert_reports_semantics("<?php enum E { public function __get(string $name): mixed {} }");
    assert_reports_semantics("<?php enum E { public function __toString(): string {} }");
}

#[test]
fn call_magic_methods_in_enum_are_ok() {
    assert_no_semantics("<?php enum E { public function __call(string $name, array $arguments): mixed {} }");
    assert_no_semantics(
        "<?php enum E { public static function __callStatic(string $name, array $arguments): mixed {} }",
    );
    assert_no_semantics("<?php enum E { public function __invoke(): mixed {} }");
}
