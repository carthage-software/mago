use bumpalo::Bump;
use mago_database::file::File;
use mago_fingerprint::FingerprintOptions;
use mago_fingerprint::Fingerprintable;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

fn fingerprint_code(code: &'static str) -> u64 {
    let arena = Bump::new();
    let file = File::ephemeral("test.php".into(), code.into());
    let (program, _) = parse_file(&arena, &file);
    let resolved_names = NameResolver::new(&arena).resolve(program);
    let options = FingerprintOptions::default();
    program.fingerprint(&resolved_names, &options)
}

#[test]
fn test_anonymous_class_with_without_parens_normalized() {
    let without_parens = fingerprint_code("<?php $a = new class {};");
    let with_parens = fingerprint_code("<?php $a = new class() {};");

    assert_eq!(
        without_parens, with_parens,
        "Anonymous class with and without parentheses should have same fingerprint"
    );
}

#[test]
fn test_regular_instantiation_with_without_parens_normalized() {
    let without_parens = fingerprint_code("<?php $a = new Foo;");
    let with_empty_parens = fingerprint_code("<?php $a = new Foo();");

    assert_eq!(
        without_parens, with_empty_parens,
        "Regular instantiation with and without empty parentheses should have same fingerprint"
    );
}

#[test]
fn test_array_syntax_normalized() {
    let short = fingerprint_code("<?php $a = [1, 2, 3];");
    let legacy = fingerprint_code("<?php $a = array(1, 2, 3);");

    assert_eq!(short, legacy, "Short array and legacy array syntax should have same fingerprint");
}

#[test]
fn test_attribute_with_without_parens() {
    let without_parens = fingerprint_code("<?php #[Foo] function bar() {}");
    let with_empty_parens = fingerprint_code("<?php #[Foo()] function bar() {}");

    assert_eq!(
        without_parens, with_empty_parens,
        "Attributes without parens vs with empty parens are semantically equivalent"
    );
}

#[test]
fn test_attribute_with_args_vs_without() {
    let without_args = fingerprint_code("<?php #[Foo()] function bar() {}");
    let with_args = fingerprint_code("<?php #[Foo(1)] function bar() {}");

    assert_ne!(without_args, with_args, "Attributes with different arguments should have different fingerprints");
}

#[test]
fn test_instantiation_with_args_vs_empty_parens() {
    let empty_parens = fingerprint_code("<?php $a = new Foo();");
    let with_args = fingerprint_code("<?php $a = new Foo(1);");

    assert_ne!(empty_parens, with_args, "Instantiation with arguments should differ from empty parentheses");
}

#[test]
fn test_instantiation_no_parens_vs_with_args() {
    let no_parens = fingerprint_code("<?php $a = new Foo;");
    let with_args = fingerprint_code("<?php $a = new Foo(1);");

    assert_ne!(no_parens, with_args, "Instantiation without parens should differ from instantiation with arguments");
}
