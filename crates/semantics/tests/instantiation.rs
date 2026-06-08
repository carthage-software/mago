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
fn class_name_followed_by_arrow_is_rejected() {
    assert_reports_semantics("<?php new Foo->bar();");
}

#[test]
fn class_name_followed_by_nullsafe_arrow_is_rejected() {
    assert_reports_semantics("<?php new Foo?->bar();");
}

#[test]
fn class_name_followed_by_class_constant_is_rejected() {
    assert_reports_semantics("<?php new Foo::BAR();");
}

#[test]
fn class_name_followed_by_bracket_is_rejected() {
    assert_reports_semantics("<?php new Foo[0]();");
}

#[test]
fn self_followed_by_arrow_is_rejected() {
    assert_reports_semantics("<?php class A { function f() { return new self->x(); } }");
}

#[test]
fn bare_class_name_is_ok() {
    assert_no_semantics("<?php new Foo;");
    assert_no_semantics("<?php new Foo();");
    assert_no_semantics("<?php new Foo(1, 2);");
}

#[test]
fn paren_wrapped_new_then_member_access_is_ok() {
    assert_no_semantics("<?php (new Foo())->bar();");
}

#[test]
fn php84_inline_new_then_member_access_is_ok() {
    assert_no_semantics("<?php new Foo()->bar();");
}

#[test]
fn variable_root_chain_is_ok() {
    assert_no_semantics("<?php new $foo();");
    assert_no_semantics("<?php new $foo->bar();");
    assert_no_semantics("<?php new $foo->bar->baz();");
    assert_no_semantics("<?php new $foo[0]();");
    assert_no_semantics("<?php new $foo::$bar();");
}

#[test]
fn class_static_property_chain_is_ok() {
    assert_no_semantics("<?php new Foo::$bar();");
    assert_no_semantics("<?php new Foo::$bar->baz();");
}

#[test]
fn parenthesized_class_expression_is_ok() {
    assert_no_semantics("<?php new ($expr)();");
}

#[test]
fn parenthesized_class_followed_by_member_access_is_rejected() {
    assert_reports_semantics("<?php new ($expr)->bar();");
}
