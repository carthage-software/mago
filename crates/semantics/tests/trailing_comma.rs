use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

fn collect_codes(source: &str, version: PHPVersion) -> Vec<String> {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(program.errors.is_empty(), "test source did not parse: {:?}", program.errors);

    let names = NameResolver::new(&arena).resolve(program);
    let issues = SemanticsChecker::new(version).check(&file, program, &names);

    issues.iter().filter_map(|issue| issue.code.clone()).collect()
}

fn assert_reports_semantics(source: &str, version: PHPVersion) {
    let codes = collect_codes(source, version);
    assert!(codes.iter().any(|c| c == "semantics"), "expected a semantics issue, got {codes:?}");
}

fn assert_no_semantics(source: &str, version: PHPVersion) {
    let codes = collect_codes(source, version);
    assert!(!codes.iter().any(|c| c == "semantics"), "did not expect a semantics issue, got {codes:?}");
}

#[test]
fn trailing_comma_in_function_parameter_list_is_rejected_before_php80() {
    assert_reports_semantics("<?php function foo($a, $b,) {}", PHPVersion::PHP74);
}

#[test]
fn trailing_comma_in_method_parameter_list_is_rejected_before_php80() {
    assert_reports_semantics("<?php class Foo { public function __construct($a, $b,) {} }", PHPVersion::PHP74);
}

#[test]
fn trailing_comma_in_closure_parameter_list_is_rejected_before_php80() {
    assert_reports_semantics("<?php $f = function ($a, $b,) {};", PHPVersion::PHP74);
}

#[test]
fn trailing_comma_in_arrow_function_parameter_list_is_rejected_before_php80() {
    assert_reports_semantics("<?php $f = fn($a, $b,) => $a;", PHPVersion::PHP74);
}

#[test]
fn trailing_comma_in_parameter_list_is_accepted_from_php80() {
    assert_no_semantics("<?php function foo($a, $b,) {}", PHPVersion::PHP80);
    assert_no_semantics("<?php class Foo { public function __construct($a, $b,) {} }", PHPVersion::PHP80);
    assert_no_semantics("<?php $f = function ($a, $b,) {};", PHPVersion::PHP80);
    assert_no_semantics("<?php $f = fn($a, $b,) => $a;", PHPVersion::PHP80);
}

#[test]
fn parameter_list_without_trailing_comma_is_accepted_before_php80() {
    assert_no_semantics("<?php function foo($a, $b) {}", PHPVersion::PHP70);
    assert_no_semantics("<?php class Foo { public function __construct($a, $b) {} }", PHPVersion::PHP70);
}
