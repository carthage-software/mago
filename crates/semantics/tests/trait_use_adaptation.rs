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
fn final_modifier_in_alias_adaptation_is_rejected() {
    assert_reports_semantics("<?php class A { use T { foo as final bar; } }");
}

#[test]
fn static_modifier_in_alias_adaptation_is_rejected() {
    assert_reports_semantics("<?php class A { use T { foo as static bar; } }");
}

#[test]
fn abstract_modifier_in_alias_adaptation_is_rejected() {
    assert_reports_semantics("<?php class A { use T { foo as abstract; } }");
}

#[test]
fn readonly_modifier_in_alias_adaptation_is_rejected() {
    assert_reports_semantics("<?php class A { use T { foo as readonly bar; } }");
}

#[test]
fn asymmetric_visibility_in_alias_adaptation_is_rejected() {
    assert_reports_semantics("<?php class A { use T { foo as protected(set) bar; } }");
}

#[test]
fn visibility_modifier_in_alias_adaptation_is_ok() {
    assert_no_semantics("<?php class A { use T { foo as public bar; } }");
    assert_no_semantics("<?php class A { use T { foo as protected bar; } }");
    assert_no_semantics("<?php class A { use T { foo as private; } }");
}

#[test]
fn alias_without_modifier_is_ok() {
    assert_no_semantics("<?php class A { use T { foo as bar; } }");
}
