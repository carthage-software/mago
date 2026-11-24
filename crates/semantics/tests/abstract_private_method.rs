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

#[test]
fn abstract_private_method_in_abstract_class_is_rejected() {
    for source in [
        "abstract class A { abstract private function f(); }",
        "abstract class A { private abstract function f(); }",
        "abstract class A { abstract private static function f(); }",
        "abstract class A { ABSTRACT PRIVATE function f(); }",
        "abstract class A { abstract private function f(int $x): string; }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP80), 1, "{source}");
    }
}

#[test]
fn abstract_private_method_is_also_rejected_next_to_other_errors() {
    for (source, version) in [
        ("class A { abstract private function f(); }", PHPVersion::PHP80),
        ("$a = new class { abstract private function f(); };", PHPVersion::PHP80),
        ("enum E { abstract private function f(); }", PHPVersion::PHP81),
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}"), version), 2, "{source}");
    }
}

#[test]
fn abstract_private_method_in_interface_reports_only_interface_errors() {
    assert_eq!(count_semantics("<?php interface I { abstract private function f(); }", PHPVersion::PHP80), 2);
}

#[test]
fn abstract_private_method_in_trait_and_other_methods_are_accepted() {
    for source in [
        "trait T { abstract private function f(); }",
        "trait T { abstract private static function f(); }",
        "abstract class A { abstract protected function f(); }",
        "abstract class A { abstract public function f(); }",
        "abstract class A { private function f() {} }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP80), 0, "{source}");
    }
}
