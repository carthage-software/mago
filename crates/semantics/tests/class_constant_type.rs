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
fn class_constant_with_invalid_type_is_rejected() {
    for source in [
        "class A { const callable X = 'strlen'; }",
        "class A { const CALLABLE X = 'strlen'; }",
        "class A { const ?callable X = null; }",
        "class A { const callable|string X = 'strlen'; }",
        "class A { const string|callable X = 'strlen'; }",
        "interface I {} interface J {} class A { const (I&J)|callable X = 'strlen'; }",
        "class A { const void X = null; }",
        "class A { const never X = null; }",
        "class A { final public const callable X = 'a', Y = 'b'; }",
        "interface I { const callable X = 'strlen'; }",
        "trait T { const never X = null; }",
        "enum E { const callable X = 'strlen'; }",
        "$a = new class { const void X = null; };",
        "class A { const void|int X = 1; }",
        "class A { const ?void X = null; }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}")), 1, "{source}");
    }
}

#[test]
fn class_constant_with_valid_type_is_accepted() {
    for source in [
        "class A { const mixed X = 'strlen'; }",
        "class A { const string X = 'strlen'; }",
        "class A { const array X = ['A', 'f']; }",
        "class A { const ?\\Closure X = null; }",
        "class A { const null X = null; }",
        "class A { const iterable X = []; }",
        "class A { const X = 'strlen'; }",
        "class A { /** @var callable */ const X = 'strlen'; }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}")), 0, "{source}");
    }
}
